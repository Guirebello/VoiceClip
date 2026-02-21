mod config;
mod audio;

use config::Config;
use audio::AudioRecorder;
use anyhow::Result;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<()> {
    println!("VoiceClip initializing...");
    
    let config = Config::load()?;
    println!("Loaded configuration: {:#?}", config);

    // Test Audio Recording
    println!("Starting audio record test in 2 seconds...");
    sleep(Duration::from_secs(2)).await;

    println!("Recording for 3 seconds... Speak now!");
    let recorder = AudioRecorder::start_recording()?;
    
    sleep(Duration::from_secs(3)).await;
    
    let save_path = "/tmp/voiceclip_audio.wav";
    recorder.stop_recording_and_save(save_path)?;
    println!("Recording saved successfully to {}", save_path);

    Ok(())
}
