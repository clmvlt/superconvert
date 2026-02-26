/// Register/unregister OS context menu entries directly from the Tauri app.
/// Uses the engine crate to enumerate supported formats, and performs
/// platform-specific operations (Windows registry, Linux scripts, macOS workflows) inline.

#[tauri::command]
pub async fn register_context_menu() -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(|| install_context_menu_entries())
        .await
        .map_err(|e| format!("Task join error: {}", e))?
}

#[tauri::command]
pub async fn unregister_context_menu() -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(|| uninstall_context_menu_entries())
        .await
        .map_err(|e| format!("Task join error: {}", e))?
}

#[tauri::command]
pub fn is_context_menu_registered() -> bool {
    #[cfg(target_os = "windows")]
    {
        use winreg::enums::*;
        use winreg::RegKey;

        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        if let Ok(classes) = hkcu.open_subkey("Software\\Classes") {
            if let Ok(ext_key) = classes.open_subkey(".png") {
                return ext_key.open_subkey("shell\\SuperConvert").is_ok();
            }
        }
        return false;
    }

    #[cfg(target_os = "linux")]
    {
        let home = std::env::var("HOME").unwrap_or_default();
        let nautilus_dir = std::path::Path::new(&home)
            .join(".local/share/nautilus/scripts/SuperConvert");
        let dolphin_file = std::path::Path::new(&home)
            .join(".local/share/kio/servicemenus/superconvert.desktop");
        return nautilus_dir.exists() || dolphin_file.exists();
    }

    #[cfg(target_os = "macos")]
    {
        let home = std::env::var("HOME").unwrap_or_default();
        let services = std::path::Path::new(&home).join("Library/Services");
        if let Ok(entries) = std::fs::read_dir(&services) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with("SuperConvert to ") && name.ends_with(".workflow") {
                    return true;
                }
            }
        }
        return false;
    }

    #[allow(unreachable_code)]
    false
}

// ---------------------------------------------------------------------------
// Platform-specific install / uninstall
// ---------------------------------------------------------------------------

#[cfg(target_os = "windows")]
fn find_cli_path() -> Result<String, String> {
    let exe = std::env::current_exe().map_err(|e| format!("Failed to get current exe: {}", e))?;
    let dir = exe.parent().ok_or("Failed to get exe directory")?;

    // In dev: superconvert-cli.exe is next to superconvert.exe
    let cli = dir.join("superconvert-cli.exe");
    if cli.exists() {
        return Ok(cli.to_string_lossy().to_string());
    }

    // Sidecar with target triple
    let sidecar = dir.join("superconvert-cli-x86_64-pc-windows-msvc.exe");
    if sidecar.exists() {
        return Ok(sidecar.to_string_lossy().to_string());
    }

    Err(format!("superconvert-cli.exe not found in {}", dir.display()))
}

#[cfg(target_os = "windows")]
fn install_context_menu_entries() -> Result<String, String> {
    use winreg::enums::*;
    use winreg::RegKey;
    use superconvert_engine::{all_supported_extensions, output_formats_for_extension};

    let cli_path = find_cli_path()?;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let classes = hkcu
        .open_subkey_with_flags("Software\\Classes", KEY_ALL_ACCESS)
        .map_err(|e| format!("Failed to open HKCU\\Software\\Classes: {}", e))?;

    let extensions = all_supported_extensions();
    let mut total_entries = 0u32;

    for ext in &extensions {
        let formats = output_formats_for_extension(ext);
        if formats.is_empty() {
            continue;
        }

        let ext_key_name = format!(".{}", ext);
        let (ext_key, _) = classes
            .create_subkey(&ext_key_name)
            .map_err(|e| format!("Failed to create key for .{}: {}", ext, e))?;

        let (sc_key, _) = ext_key
            .create_subkey("shell\\SuperConvert")
            .map_err(|e| format!("Failed to create SuperConvert key for .{}: {}", ext, e))?;

        sc_key.set_value("", &"Convert with SuperConvert")
            .map_err(|e| format!("Registry set_value error: {}", e))?;
        sc_key.set_value("SubCommands", &"")
            .map_err(|e| format!("Registry set_value error: {}", e))?;

        for fmt in &formats {
            let action_name = format!("To{}", fmt.label);
            let (action_key, _) = sc_key
                .create_subkey(format!("shell\\{}", action_name))
                .map_err(|e| format!("Failed to create action key {}: {}", action_name, e))?;

            action_key.set_value("", &fmt.label)
                .map_err(|e| format!("Registry set_value error: {}", e))?;

            let command = format!(
                "\"{}\" convert \"%1\" --to {}",
                cli_path, fmt.format
            );
            let (cmd_key, _) = action_key
                .create_subkey("command")
                .map_err(|e| format!("Failed to create command key: {}", e))?;
            cmd_key.set_value("", &command)
                .map_err(|e| format!("Registry set_value error: {}", e))?;

            total_entries += 1;
        }
    }

    Ok(format!("Installed {} context menu entries for {} extensions.", total_entries, extensions.len()))
}

#[cfg(target_os = "windows")]
fn uninstall_context_menu_entries() -> Result<String, String> {
    use winreg::enums::*;
    use winreg::RegKey;
    use superconvert_engine::all_supported_extensions;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let classes = match hkcu.open_subkey_with_flags("Software\\Classes", KEY_ALL_ACCESS) {
        Ok(k) => k,
        Err(_) => return Ok("No context menu entries found.".to_string()),
    };

    let extensions = all_supported_extensions();
    let mut removed = 0u32;

    for ext in &extensions {
        let ext_key_name = format!(".{}", ext);
        let ext_key = match classes.open_subkey_with_flags(&ext_key_name, KEY_ALL_ACCESS) {
            Ok(k) => k,
            Err(_) => continue,
        };

        if delete_registry_tree(&ext_key, "shell\\SuperConvert").is_ok() {
            removed += 1;
        }
    }

    Ok(format!("Removed context menu entries for {} extensions.", removed))
}

#[cfg(target_os = "windows")]
fn delete_registry_tree(parent: &winreg::RegKey, subkey: &str) -> Result<(), String> {
    use winreg::enums::*;

    if let Ok(key) = parent.open_subkey_with_flags(subkey, KEY_ALL_ACCESS) {
        let subkey_names: Vec<String> = key.enum_keys().filter_map(|k| k.ok()).collect();
        for child_name in subkey_names {
            delete_registry_tree(&key, &child_name)?;
        }
    }

    parent
        .delete_subkey_all(subkey)
        .map_err(|e| format!("Failed to delete registry key {}: {}", subkey, e))
}

// Linux stubs
#[cfg(target_os = "linux")]
fn install_context_menu_entries() -> Result<String, String> {
    Err("Linux context menu install not yet implemented in Tauri commands".to_string())
}

#[cfg(target_os = "linux")]
fn uninstall_context_menu_entries() -> Result<String, String> {
    Err("Linux context menu uninstall not yet implemented in Tauri commands".to_string())
}

// macOS stubs
#[cfg(target_os = "macos")]
fn install_context_menu_entries() -> Result<String, String> {
    Err("macOS context menu install not yet implemented in Tauri commands".to_string())
}

#[cfg(target_os = "macos")]
fn uninstall_context_menu_entries() -> Result<String, String> {
    Err("macOS context menu uninstall not yet implemented in Tauri commands".to_string())
}

// Fallback for other platforms
#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
fn install_context_menu_entries() -> Result<String, String> {
    Err("Context menu integration is not supported on this platform".to_string())
}

#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
fn uninstall_context_menu_entries() -> Result<String, String> {
    Err("Context menu integration is not supported on this platform".to_string())
}
