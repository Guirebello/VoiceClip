use anyhow::Result;

pub fn copy_to_clipboard(text: &str, append_mode: bool) -> Result<()> {
    let mut clipboard = arboard::Clipboard::new()?;
    if append_mode {
        if let Ok(current) = clipboard.get_text() {
            if !current.trim().is_empty() {
                clipboard.set_text(format!("{} {}", current.trim(), text))?;
                return Ok(());
            }
        }
    }
    clipboard.set_text(text.to_string())?;
    Ok(())
}

pub fn notify(title: &str, body: &str, _is_error: bool) -> Result<()> {
    let mut notification = notify_rust::Notification::new();
    notification.summary(title).body(body);
    #[cfg(target_os = "linux")]
    {
        let urgency = if _is_error {
            notify_rust::Urgency::Critical
        } else {
            notify_rust::Urgency::Normal
        };
        notification.urgency(urgency);
    }
    notification.show()?;
    Ok(())
}
