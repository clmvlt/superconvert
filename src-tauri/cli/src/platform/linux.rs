use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use anyhow::{Context, Result};

use superconvert_engine::{all_supported_extensions, output_formats_for_extension, file_category};

fn cli_exe_path() -> Result<String> {
    let exe = std::env::current_exe().context("Failed to get current exe path")?;
    Ok(exe.to_string_lossy().to_string())
}

fn nautilus_scripts_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(home).join(".local/share/nautilus/scripts/SuperConvert")
}

fn dolphin_service_menu_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(home).join(".local/share/kio/servicemenus/superconvert.desktop")
}

/// Install context menu entries for Nautilus and Dolphin.
pub fn install() -> Result<()> {
    let cli_path = cli_exe_path()?;

    install_nautilus_scripts(&cli_path)?;
    install_dolphin_service_menu(&cli_path)?;

    println!("Context menu installed for Nautilus and Dolphin.");
    Ok(())
}

/// Remove context menu entries.
pub fn uninstall() -> Result<()> {
    // Remove Nautilus scripts
    let scripts_dir = nautilus_scripts_dir();
    if scripts_dir.exists() {
        fs::remove_dir_all(&scripts_dir)
            .context("Failed to remove Nautilus scripts")?;
    }

    // Remove Dolphin service menu
    let desktop_file = dolphin_service_menu_path();
    if desktop_file.exists() {
        fs::remove_file(&desktop_file)
            .context("Failed to remove Dolphin service menu")?;
    }

    println!("Context menu entries removed.");
    Ok(())
}

/// Check if context menu is installed.
pub fn is_installed() -> bool {
    nautilus_scripts_dir().exists() || dolphin_service_menu_path().exists()
}

/// Install Nautilus scripts organized by category.
fn install_nautilus_scripts(cli_path: &str) -> Result<()> {
    let base_dir = nautilus_scripts_dir();
    let _ = fs::remove_dir_all(&base_dir);
    fs::create_dir_all(&base_dir)?;

    let categories = ["image", "audio", "video", "document", "spreadsheet", "presentation", "data", "archive"];

    for category in &categories {
        // Find extensions belonging to this category
        let exts: Vec<&&str> = all_supported_extensions()
            .iter()
            .filter(|e| file_category(e) == Some(category))
            .collect();

        if exts.is_empty() {
            continue;
        }

        // Collect unique output formats for this category
        let mut seen_formats = std::collections::HashSet::new();
        let mut formats = Vec::new();
        for ext in &exts {
            for fmt in output_formats_for_extension(ext) {
                if seen_formats.insert(fmt.format.clone()) {
                    formats.push(fmt);
                }
            }
        }

        if formats.is_empty() {
            continue;
        }

        let cat_label = capitalize(category);
        let cat_dir = base_dir.join(&cat_label);
        fs::create_dir_all(&cat_dir)?;

        for fmt in &formats {
            let script_path = cat_dir.join(format!("to {}", fmt.label));
            let script_content = format!(
                "#!/bin/bash\nwhile IFS= read -r file; do\n    \"{}\" convert \"$file\" --to {}\ndone <<< \"$NAUTILUS_SCRIPT_SELECTED_FILE_PATHS\"\n",
                cli_path, fmt.format
            );
            fs::write(&script_path, script_content)?;
            fs::set_permissions(&script_path, fs::Permissions::from_mode(0o755))?;
        }
    }

    Ok(())
}

/// Install Dolphin (KDE) service menu.
fn install_dolphin_service_menu(cli_path: &str) -> Result<()> {
    let desktop_path = dolphin_service_menu_path();
    if let Some(parent) = desktop_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut mime_types = Vec::new();
    let categories_mime = [
        ("image", "image/*"),
        ("audio", "audio/*"),
        ("video", "video/*"),
        ("document", "application/pdf"),
        ("archive", "application/zip"),
    ];
    for (_, mime) in &categories_mime {
        mime_types.push(*mime);
    }

    // Collect all unique formats
    let mut all_formats = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for ext in all_supported_extensions() {
        for fmt in output_formats_for_extension(ext) {
            if seen.insert(fmt.format.clone()) {
                all_formats.push(fmt);
            }
        }
    }

    let actions: Vec<String> = all_formats
        .iter()
        .map(|f| format!("To{}", f.label))
        .collect();

    let mut content = format!(
        "[Desktop Entry]\nType=Service\nMimeType={}\nX-KDE-Submenu=Convert with SuperConvert\nActions={}\n\n",
        mime_types.join(";") + ";",
        actions.join(";")
    );

    for fmt in &all_formats {
        content.push_str(&format!(
            "[Desktop Action To{}]\nName={}\nExec=\"{}\" convert %f --to {}\n\n",
            fmt.label, fmt.label, cli_path, fmt.format
        ));
    }

    fs::write(&desktop_path, content)?;

    Ok(())
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().to_string() + chars.as_str(),
    }
}
