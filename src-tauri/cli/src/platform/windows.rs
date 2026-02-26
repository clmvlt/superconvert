use anyhow::{Context, Result};
use winreg::enums::*;
use winreg::RegKey;

use superconvert_engine::{all_supported_extensions, output_formats_for_extension};

/// Get the path to the current CLI executable.
fn cli_exe_path() -> Result<String> {
    let exe = std::env::current_exe().context("Failed to get current exe path")?;
    Ok(exe.to_string_lossy().to_string())
}

/// Install context menu entries for all supported extensions under HKCU.
///
/// Registry structure (per extension):
/// ```text
/// HKCU\Software\Classes\.png\shell\SuperConvert\
///     (Default) = "Convert with SuperConvert"
///     SubCommands = ""
///     shell\
///         ToJPG\
///             (Default) = "JPG"
///             command\ = "...\superconvert-cli.exe" convert "%1" --to jpg
///         ToWebP\
///             ...
/// ```
pub fn install() -> Result<()> {
    let cli_path = cli_exe_path()?;
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let classes = hkcu.open_subkey_with_flags("Software\\Classes", KEY_ALL_ACCESS)
        .context("Failed to open HKCU\\Software\\Classes")?;

    let extensions = all_supported_extensions();
    let mut total_entries = 0u32;

    for ext in &extensions {
        let formats = output_formats_for_extension(ext);
        if formats.is_empty() {
            continue;
        }

        let ext_key_name = format!(".{}", ext);

        // Create or open the extension key
        let (ext_key, _) = classes
            .create_subkey(&ext_key_name)
            .with_context(|| format!("Failed to create key for .{}", ext))?;

        // Create shell\SuperConvert
        let (sc_key, _) = ext_key
            .create_subkey("shell\\SuperConvert")
            .with_context(|| format!("Failed to create SuperConvert key for .{}", ext))?;

        sc_key.set_value("", &"Convert with SuperConvert")?;
        sc_key.set_value("SubCommands", &"")?;

        // Create sub-commands for each output format
        for fmt in &formats {
            let action_name = format!("To{}", fmt.label);
            let (action_key, _) = sc_key
                .create_subkey(format!("shell\\{}", action_name))
                .with_context(|| format!("Failed to create action key {}", action_name))?;

            action_key.set_value("", &fmt.label)?;

            let command = format!(
                "\"{}\" convert \"%1\" --to {}",
                cli_path, fmt.format
            );
            let (cmd_key, _) = action_key.create_subkey("command")?;
            cmd_key.set_value("", &command)?;

            total_entries += 1;
        }
    }

    println!("Installed {} context menu entries for {} extensions.", total_entries, extensions.len());
    Ok(())
}

/// Remove all SuperConvert context menu entries.
pub fn uninstall() -> Result<()> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let classes = match hkcu.open_subkey_with_flags("Software\\Classes", KEY_ALL_ACCESS) {
        Ok(k) => k,
        Err(_) => {
            println!("No context menu entries found.");
            return Ok(());
        }
    };

    let extensions = all_supported_extensions();
    let mut removed = 0u32;

    for ext in &extensions {
        let ext_key_name = format!(".{}", ext);

        let ext_key = match classes.open_subkey_with_flags(&ext_key_name, KEY_ALL_ACCESS) {
            Ok(k) => k,
            Err(_) => continue,
        };

        // Try to delete shell\SuperConvert and all sub-keys
        if delete_tree(&ext_key, "shell\\SuperConvert").is_ok() {
            removed += 1;
        }
    }

    println!("Removed context menu entries for {} extensions.", removed);
    Ok(())
}

/// Check if context menu entries are installed by looking for any SuperConvert key.
#[allow(dead_code)]
pub fn is_installed() -> bool {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let classes = match hkcu.open_subkey("Software\\Classes") {
        Ok(k) => k,
        Err(_) => return false,
    };

    // Check a common extension (e.g., .png)
    let ext_key = match classes.open_subkey(".png") {
        Ok(k) => k,
        Err(_) => return false,
    };

    ext_key.open_subkey("shell\\SuperConvert").is_ok()
}

/// Recursively delete a registry key and all its sub-keys.
fn delete_tree(parent: &RegKey, subkey: &str) -> Result<()> {
    // First, recursively delete children
    if let Ok(key) = parent.open_subkey_with_flags(subkey, KEY_ALL_ACCESS) {
        let subkey_names: Vec<String> = key.enum_keys().filter_map(|k| k.ok()).collect();
        for child_name in subkey_names {
            delete_tree(&key, &child_name)?;
        }
    }

    parent
        .delete_subkey_all(subkey)
        .with_context(|| format!("Failed to delete registry key: {}", subkey))?;

    Ok(())
}
