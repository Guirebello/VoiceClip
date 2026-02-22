use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct Config {
    pub model_name: String,
    pub hotkey: String,
    pub badge_opacity: f32,
    pub max_recording_duration: u32,
    pub append_mode: bool,
    pub microphone: Option<String>,
    pub always_on_top: bool,
    pub badge_x: Option<i32>,
    pub badge_y: Option<i32>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            model_name: "base.en".to_string(),
            hotkey: "Super+Alt+V".to_string(),
            badge_opacity: 0.8,
            max_recording_duration: 120,
            append_mode: false,
            microphone: None,
            always_on_top: true,
            badge_x: None,
            badge_y: None,
        }
    }
}

impl Config {
    pub fn get_config_dir() -> Result<PathBuf> {
        let proj_dirs = ProjectDirs::from("", "", "voiceclip")
            .context("Could not find project directories")?;
        Ok(proj_dirs.config_dir().to_path_buf())
    }

    pub fn get_models_dir() -> Result<PathBuf> {
        if let Ok(exe) = std::env::current_exe() {
            if let Some(dir) = exe.parent() {
                let bundled_models = dir.join("models");
                if bundled_models.exists() {
                    return Ok(bundled_models);
                }
            }
        }
        let proj_dirs = ProjectDirs::from("", "", "voiceclip")
            .context("Could not find project directories")?;
        Ok(proj_dirs.data_local_dir().join("models"))
    }

    pub fn get_db_path() -> Result<PathBuf> {
        let proj_dirs = ProjectDirs::from("", "", "voiceclip")
            .context("Could not find project directories")?;
        Ok(proj_dirs.data_local_dir().join("voiceclip.db"))
    }

    pub fn load() -> Result<Self> {
        let config_dir = Self::get_config_dir()?;
        let config_path = config_dir.join("config.toml");

        if !config_path.exists() {
            let default_config = Self::default();
            default_config.save()?;
            return Ok(default_config);
        }

        let contents = fs::read_to_string(&config_path)
            .with_context(|| format!("Failed to read config file at {:?}", config_path))?;

        let config: Config = toml::from_str(&contents)
            .with_context(|| format!("Failed to parse config file at {:?}", config_path))?;

        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let config_dir = Self::get_config_dir()?;
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)
                .with_context(|| format!("Failed to create config directory at {:?}", config_dir))?;
        }

        let config_path = config_dir.join("config.toml");
        let toml_string = toml::to_string_pretty(self)
            .context("Failed to serialize config to TOML")?;

        fs::write(&config_path, toml_string)
            .with_context(|| format!("Failed to write config file to {:?}", config_path))?;

        Ok(())
    }
}
