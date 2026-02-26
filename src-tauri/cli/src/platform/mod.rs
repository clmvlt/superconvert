#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;

use anyhow::Result;

pub fn install_context_menu() -> Result<()> {
    #[cfg(target_os = "windows")]
    return windows::install();

    #[cfg(target_os = "linux")]
    return linux::install();

    #[cfg(target_os = "macos")]
    return macos::install();

    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        anyhow::bail!("Context menu integration is not supported on this platform");
    }
}

pub fn uninstall_context_menu() -> Result<()> {
    #[cfg(target_os = "windows")]
    return windows::uninstall();

    #[cfg(target_os = "linux")]
    return linux::uninstall();

    #[cfg(target_os = "macos")]
    return macos::uninstall();

    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        anyhow::bail!("Context menu integration is not supported on this platform");
    }
}
