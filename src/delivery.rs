use anyhow::{Context, Result};
use tokio::process::Command;
use std::process::Stdio;

pub async fn copy_to_clipboard(text: &str, append_mode: bool) -> Result<()> {
    let mut final_text = text.to_string();

    if append_mode {
        // Read current clipboard
        let paste_output = Command::new("wl-paste")
            .output()
            .await;

        if let Ok(output) = paste_output {
            if output.status.success() {
                let current_clip = String::from_utf8_lossy(&output.stdout);
                if !current_clip.trim().is_empty() {
                    final_text = format!("{} {}", current_clip.trim_end(), text);
                }
            }
        }
    }

    let mut child = Command::new("wl-copy")
        .stdin(Stdio::piped())
        .spawn()
        .context("Failed to spawn wl-copy. Is it installed?")?;

    if let Some(mut stdin) = child.stdin.take() {
        use tokio::io::AsyncWriteExt;
        stdin.write_all(final_text.as_bytes())
            .await
            .context("Failed to write to wl-copy stdin")?;
    }

    let status = child.wait().await.context("Failed to wait on wl-copy")?;
    if !status.success() {
        anyhow::bail!("wl-copy returned non-zero exit status: {}", status);
    }

    Ok(())
}

pub async fn notify(title: &str, body: &str, is_error: bool) -> Result<()> {
    let urgency = if is_error { "critical" } else { "normal" };

    let status = Command::new("notify-send")
        .arg("-u")
        .arg(urgency)
        .arg("-a")
        .arg("VoiceClip")
        .arg(title)
        .arg(body)
        .status()
        .await
        .context("Failed to execute notify-send")?;

    if !status.success() {
        anyhow::bail!("notify-send returned non-zero exit status: {}", status);
    }

    Ok(())
}
