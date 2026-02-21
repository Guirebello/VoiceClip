use std::path::Path;
use tokio::process::Command;
use anyhow::{Context, Result};

pub async fn transcribe(wav_path: &Path, model_path: &Path) -> Result<String> {
    // whisper-cli -m model.bin -f file.wav --output-txt --no-timestamps -l auto
    let output = Command::new("whisper-cli")
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
        anyhow::bail!("whisper-cli failed to execute:\n{}", stderr);
    }

    // --output-txt produces a file with the same name + .txt extension
    // Wait, the prompt says "Wait for the process to finish, then read the generated output/text and return it as a cleaned-up String."
    // Let's check how --output-txt works in whisper.cpp. It generates a .txt file named like the input file + ".txt"
    // E.g. /tmp/voiceclip_audio.wav -> /tmp/voiceclip_audio.wav.txt
    
    // Instead of stdout, check the .txt file.
    let txt_path = wav_path.with_extension("wav.txt"); // input is .wav, output is .wav.txt typically.
    
    // Some versions output exactly to stdout or create a specific txt file.
    // Let's assume whisper.cpp standard behaviour. `whisper-cli -f file.wav --output-txt` writes file.wav.txt.
    let text = tokio::fs::read_to_string(&txt_path)
        .await
        .unwrap_or_else(|_| String::from_utf8_lossy(&output.stdout).to_string());

    // Clean up
    let text = text.trim().to_string();

    Ok(text)
}
