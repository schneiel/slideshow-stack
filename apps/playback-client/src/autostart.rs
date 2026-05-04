use video_rs::path_utils::validate_filenames;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use tracing::{info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutostartConfig {
    pub enabled: bool,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub media_files: Vec<String>,
    #[serde(default)]
    #[serde(rename = "interval_seconds")]
    pub interval_seconds: f64,
    #[serde(default)]
    pub options: AutostartOptions,
    #[serde(default)]
    #[serde(rename = "scaling_mode")]
    pub scaling_mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutostartOptions {
    #[serde(default)]
    #[serde(rename = "shuffle_enabled")]
    pub shuffle_enabled: bool,
    #[serde(default)]
    #[serde(rename = "loop_enabled")]
    pub loop_enabled: bool,
}

impl Default for AutostartConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            name: "autostart-slideshow".to_string(),
            media_files: Vec::new(),
            interval_seconds: 5.0,
            options: AutostartOptions::default(),
            scaling_mode: "fit".to_string(),
        }
    }
}

impl Default for AutostartOptions {
    fn default() -> Self {
        Self {
            shuffle_enabled: false,
            loop_enabled: true,
        }
    }
}

pub fn load_autostart_config(config_path: &str) -> Result<AutostartConfig> {
    let path = Path::new(config_path);

    if !path.exists() {
        info!("Autostart config file not found, using defaults");
        return Ok(AutostartConfig::default());
    }

    let content = fs::read_to_string(path).context("Failed to read autostart config file")?;

    let mut config: AutostartConfig = match serde_json::from_str(&content) {
        Ok(cfg) => cfg,
        Err(e) => {
            warn!("Invalid autostart config JSON: {}, using defaults", e);
            return Ok(AutostartConfig::default());
        }
    };

    validate_autostart_config(&config)?;

    if config.name.is_empty() {
        config.name = "autostart-slideshow".to_string();
    }
    if config.scaling_mode.is_empty() {
        config.scaling_mode = "fit".to_string();
    }

    Ok(config)
}

fn validate_autostart_config(config: &AutostartConfig) -> Result<()> {
    if config.enabled {
        if config.media_files.is_empty() {
            info!("Autostart enabled with no media files - will fetch from server");
        } else {
            validate_filenames(&config.media_files)
                .context("Invalid filename in autostart config")?;
        }
    }

    Ok(())
}

pub fn save_autostart_config(config_path: &str, config: &AutostartConfig) -> Result<()> {
    let path = Path::new(config_path);

    validate_autostart_config(config)?;

    let json =
        serde_json::to_string_pretty(config).context("Failed to serialize autostart config")?;

    fs::write(path, json).context("Failed to write autostart config file")?;

    info!("Autostart config saved to {}", config_path);
    Ok(())
}
