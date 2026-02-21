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

pub fn notify(title: &str, body: &str, is_error: bool) -> Result<()> {
    let urgency = if is_error {
        notify_rust::Urgency::Critical
    } else {
        notify_rust::Urgency::Normal
    };
    notify_rust::Notification::new()
        .summary(title)
        .body(body)
        .urgency(urgency)
        .show()?;
    Ok(())
}
