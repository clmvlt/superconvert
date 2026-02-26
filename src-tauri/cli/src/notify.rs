/// Show a system notification (best-effort, never panics).
pub fn show_notification(title: &str, message: &str, is_error: bool) {
    // Always print to console
    if is_error {
        eprintln!("{}: {}", title, message);
    } else {
        println!("{}: {}", title, message);
    }

    // Platform-specific desktop notification
    #[cfg(target_os = "windows")]
    show_notification_windows(title, message);

    #[cfg(target_os = "linux")]
    show_notification_linux(title, message);

    #[cfg(target_os = "macos")]
    show_notification_macos(title, message);
}

#[cfg(target_os = "windows")]
fn show_notification_windows(title: &str, message: &str) {
    use std::os::windows::process::CommandExt;

    // Use PowerShell to show a toast notification
    let script = format!(
        "[Windows.UI.Notifications.ToastNotificationManager, Windows.UI.Notifications, ContentType = WindowsRuntime] | Out-Null; \
         [Windows.Data.Xml.Dom.XmlDocument, Windows.Data.Xml.Dom, ContentType = WindowsRuntime] | Out-Null; \
         $template = '<toast><visual><binding template=\"ToastText02\"><text id=\"1\">{}</text><text id=\"2\">{}</text></binding></visual></toast>'; \
         $xml = New-Object Windows.Data.Xml.Dom.XmlDocument; \
         $xml.LoadXml($template); \
         $toast = [Windows.UI.Notifications.ToastNotification]::new($xml); \
         [Windows.UI.Notifications.ToastNotificationManager]::CreateToastNotifier('SuperConvert').Show($toast)",
        escape_xml(title),
        escape_xml(message)
    );

    let _ = std::process::Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &script])
        .creation_flags(0x08000000) // CREATE_NO_WINDOW
        .spawn();
}

#[cfg(target_os = "linux")]
fn show_notification_linux(title: &str, message: &str) {
    let _ = std::process::Command::new("notify-send")
        .args([title, message])
        .spawn();
}

#[cfg(target_os = "macos")]
fn show_notification_macos(title: &str, message: &str) {
    let script = format!(
        "display notification \"{}\" with title \"{}\"",
        message.replace('"', "\\\""),
        title.replace('"', "\\\"")
    );
    let _ = std::process::Command::new("osascript")
        .args(["-e", &script])
        .spawn();
}

#[cfg(target_os = "windows")]
fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
