use std::path::Path;
use tokio::process::Command;
use anyhow::{Context, Result};

/// Resolve whisper-cli path: check next to our executable first, then fall back to PATH.
fn get_whisper_cli_path() -> String {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let bundled = dir.join(if cfg!(windows) { "whisper-cli.exe" } else { "whisper-cli" });
            if bundled.exists() {
                return bundled.to_string_lossy().to_string();
            }
        }
    }
    "whisper-cli".to_string()
}

pub async fn transcribe(wav_path: &Path, model_path: &Path) -> Result<String> {
    let output = Command::new(get_whisper_cli_path())
        .arg("-m")
        .arg(model_path)
        .arg("-f")
        .arg(wav_path)
        .arg("--output-txt")
        .arg("--no-timestamps")
        .arg("-l")
        .arg("auto")
        .output()
        .await
        .context("Failed to execute whisper-cli. Is it installed and in PATH?")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut msg = format!("whisper-cli exited with {}", output.status);
        if !stderr.is_empty() {
            msg.push_str(&format!("\nstderr:\n{}", stderr));
        }
        if !stdout.is_empty() {
            msg.push_str(&format!("\nstdout:\n{}", stdout));
        }
        anyhow::bail!("{}", msg);
    }

    let txt_path = wav_path.with_extension("wav.txt");

    let text = tokio::fs::read_to_string(&txt_path)
        .await
        .unwrap_or_else(|_| String::from_utf8_lossy(&output.stdout).to_string());

    let text = text.trim().to_string();

    Ok(text)
}
