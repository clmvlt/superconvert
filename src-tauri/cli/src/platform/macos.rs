use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};

use superconvert_engine::{all_supported_extensions, output_formats_for_extension, file_category};

fn cli_exe_path() -> Result<String> {
    let exe = std::env::current_exe().context("Failed to get current exe path")?;
    Ok(exe.to_string_lossy().to_string())
}

fn services_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(home).join("Library/Services")
}

/// Install macOS Quick Action workflows.
pub fn install() -> Result<()> {
    let cli_path = cli_exe_path()?;
    let services = services_dir();
    fs::create_dir_all(&services)?;

    // Collect unique formats
    let mut seen = std::collections::HashSet::new();
    let mut formats = Vec::new();
    for ext in all_supported_extensions() {
        for fmt in output_formats_for_extension(ext) {
            if seen.insert(fmt.format.clone()) {
                formats.push(fmt);
            }
        }
    }

    for fmt in &formats {
        let workflow_name = format!("SuperConvert to {}.workflow", fmt.label);
        let workflow_dir = services.join(&workflow_name).join("Contents");
        fs::create_dir_all(&workflow_dir)?;

        let plist = generate_workflow_plist(&cli_path, &fmt.format);
        fs::write(workflow_dir.join("document.wflow"), plist)?;
    }

    println!("Installed {} Quick Action workflows.", formats.len());
    Ok(())
}

/// Remove all SuperConvert Quick Action workflows.
pub fn uninstall() -> Result<()> {
    let services = services_dir();
    if !services.exists() {
        println!("No Quick Action workflows found.");
        return Ok(());
    }

    let mut removed = 0u32;
    for entry in fs::read_dir(&services)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with("SuperConvert to ") && name.ends_with(".workflow") {
            fs::remove_dir_all(entry.path())?;
            removed += 1;
        }
    }

    println!("Removed {} Quick Action workflows.", removed);
    Ok(())
}

/// Check if any SuperConvert workflows are installed.
pub fn is_installed() -> bool {
    let services = services_dir();
    if !services.exists() {
        return false;
    }

    if let Ok(entries) = fs::read_dir(&services) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("SuperConvert to ") && name.ends_with(".workflow") {
                return true;
            }
        }
    }

    false
}

/// Generate an Automator workflow plist that runs the CLI command.
fn generate_workflow_plist(cli_path: &str, format: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>AMApplicationBuild</key>
    <string>523</string>
    <key>AMApplicationVersion</key>
    <string>2.10</string>
    <key>AMDocumentVersion</key>
    <string>2</string>
    <key>actions</key>
    <array>
        <dict>
            <key>action</key>
            <dict>
                <key>AMAccepts</key>
                <dict>
                    <key>Container</key>
                    <string>List</string>
                    <key>Optional</key>
                    <true/>
                    <key>Types</key>
                    <array>
                        <string>com.apple.cocoa.path</string>
                    </array>
                </dict>
                <key>AMActionVersion</key>
                <string>1.0.2</string>
                <key>AMApplication</key>
                <array>
                    <string>Automator</string>
                </array>
                <key>AMBundleIdentifier</key>
                <string>com.apple.RunShellScript</string>
                <key>AMCategory</key>
                <array>
                    <string>AMCategoryUtilities</string>
                </array>
                <key>AMIconName</key>
                <string>Automator</string>
                <key>AMKeywords</key>
                <array>
                    <string>Shell</string>
                    <string>Script</string>
                </array>
                <key>AMName</key>
                <string>Run Shell Script</string>
                <key>AMParameters</key>
                <dict>
                    <key>COMMAND_STRING</key>
                    <string>for f in "$@"; do "{}" convert "$f" --to {}; done</string>
                    <key>CheckedForUserDefaultShell</key>
                    <true/>
                    <key>inputMethod</key>
                    <integer>1</integer>
                    <key>shell</key>
                    <string>/bin/bash</string>
                    <key>source</key>
                    <string></string>
                </dict>
                <key>AMProvides</key>
                <dict>
                    <key>Container</key>
                    <string>List</string>
                    <key>Types</key>
                    <array>
                        <string>com.apple.cocoa.path</string>
                    </array>
                </dict>
                <key>AMRequiredResources</key>
                <array/>
                <key>ActionBundlePath</key>
                <string>/System/Library/Automator/Run Shell Script.action</string>
                <key>ActionName</key>
                <string>Run Shell Script</string>
                <key>ActionParameters</key>
                <dict/>
                <key>BundleIdentifier</key>
                <string>com.apple.RunShellScript</string>
                <key>CFBundleVersion</key>
                <string>1.0.2</string>
                <key>CanShowSelectedItemsWhenRun</key>
                <false/>
                <key>CanShowWhenRun</key>
                <true/>
                <key>Category</key>
                <array>
                    <string>AMCategoryUtilities</string>
                </array>
                <key>Class Name</key>
                <string>RunShellScriptAction</string>
                <key>InputUUID</key>
                <string>E1A26C46-1234-4567-8901-ABCDEF012345</string>
                <key>Keywords</key>
                <array>
                    <string>Shell</string>
                    <string>Script</string>
                </array>
                <key>OutputUUID</key>
                <string>F2B37D57-2345-5678-9012-BCDEF0123456</string>
                <key>UUID</key>
                <string>A3C48E68-3456-6789-0123-CDEF01234567</string>
                <key>UnlocalizedApplications</key>
                <array>
                    <string>Automator</string>
                </array>
            </dict>
        </dict>
    </array>
    <key>connectors</key>
    <dict/>
    <key>workflowMetaData</key>
    <dict>
        <key>applicationBundleIDsByPath</key>
        <dict/>
        <key>applicationPaths</key>
        <array/>
        <key>inputTypeIdentifier</key>
        <string>com.apple.Automator.fileSystemObject</string>
        <key>outputTypeIdentifier</key>
        <string>com.apple.Automator.nothing</string>
        <key>presentationMode</key>
        <integer>15</integer>
        <key>processesInput</key>
        <integer>0</integer>
        <key>serviceApplicationBundleID</key>
        <string>com.apple.finder</string>
        <key>serviceApplicationPath</key>
        <string>/System/Applications/Finder.app</string>
        <key>serviceInputTypeIdentifier</key>
        <string>com.apple.Automator.fileSystemObject</string>
        <key>serviceOutputTypeIdentifier</key>
        <string>com.apple.Automator.nothing</string>
        <key>serviceProcessesInput</key>
        <integer>0</integer>
        <key>systemImageName</key>
        <string>NSActionTemplate</string>
        <key>useAutomaticInputType</key>
        <integer>0</integer>
        <key>workflowTypeIdentifier</key>
        <string>com.apple.Automator.servicesMenu</string>
    </dict>
</dict>
</plist>"#,
        cli_path, format
    )
}
