mod config;

use config::Config;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    println!("VoiceClip initializing...");
    
    let config = Config::load()?;
    println!("Loaded configuration: {:#?}", config);

    Ok(())
}
