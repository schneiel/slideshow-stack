use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::slideshow_executor::{Command as SlideshowCommand, StartSlideshowData};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZenohCommand {
    #[serde(rename = "device_id")]
    pub device_id: String,
    pub timestamp: i64,
    pub command: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<serde_json::Value>,
    pub broadcast: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SlideshowState {
    pub status: String,
    pub name: Option<String>,
    pub image: Option<String>,
    pub interval: Option<i64>,
    pub scaling_mode: Option<String>,
    pub total_images: Option<i64>,
    pub current_index: Option<i64>,
}

impl Default for SlideshowState {
    fn default() -> Self {
        Self {
            status: "stopped".to_string(),
            name: None,
            image: None,
            interval: None,
            scaling_mode: None,
            total_images: None,
            current_index: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VideoState {
    pub status: String,
    pub filename: Option<String>,
    pub current_frame: Option<u32>,
    pub total_frames: Option<u32>,
    pub scaling_mode: Option<String>,
}

impl Default for VideoState {
    fn default() -> Self {
        Self {
            status: "idle".to_string(),
            filename: None,
            current_frame: None,
            total_frames: None,
            scaling_mode: None,
        }
    }
}



pub fn parse_command_impl(
    command: &str,
    config: Option<serde_json::Value>,
    media_directory: &str,
    autostart_file: &str,
) -> Result<SlideshowCommand> {
    let command = match command {
        "start" => {
            let data_obj = config.ok_or_else(|| anyhow!("Missing config for start"))?;

            let start_data: serde_json::Value = if data_obj.is_array() {
                json!({
                    "name": "Slideshow",
                    "interval_seconds": 5.0,
                    "shuffle_enabled": false,
                    "loop_enabled": true,
                    "scaling_mode": 1,
                    "media_files": data_obj,
                    "media_directory": media_directory,
                })
            } else {
                let mut data_obj = data_obj;
                if let Some(obj) = data_obj.as_object_mut()
                    && !obj.contains_key("media_directory")
                {
                    obj.insert("media_directory".to_string(), json!(media_directory));
                }
                data_obj
            };

            let slideshow_data: StartSlideshowData = serde_json::from_value(start_data)
                .context("Failed to parse StartSlideshow data")?;

            SlideshowCommand::StartSlideshow {
                data: slideshow_data,
            }
        }

        "stop_slideshow" => SlideshowCommand::StopSlideshow,

        "pause_slideshow" => SlideshowCommand::PauseSlideshow,

        "resume_slideshow" => SlideshowCommand::ResumeSlideshow,

        "next" => SlideshowCommand::NextImage,

        "previous" => SlideshowCommand::PreviousImage,

        "set_scaling_mode" => {
            let config_obj =
                config.ok_or_else(|| anyhow!("Missing config for set_scaling_mode"))?;
            let mode_value = config_obj
                .get("mode")
                .and_then(serde_json::Value::as_i64)
                .ok_or_else(|| anyhow!("Invalid mode"))?;
            let mode = mode_value
                .try_into()
                .map_err(|_| anyhow!("scaling mode value {mode_value} out of range for i32"))?;

            SlideshowCommand::SetScalingMode { mode }
        }

        "set_config" => {
            let config_obj = config.ok_or_else(|| anyhow!("Missing config for set_config"))?;
            let autostart_config: crate::autostart::AutostartConfig =
                serde_json::from_value(config_obj).context("Failed to parse autostart config")?;

            crate::autostart::save_autostart_config(autostart_file, &autostart_config)
                .context("Failed to save autostart config")?;

            tracing::info!(
                "Autostart config saved to {} (enabled: {})",
                autostart_file,
                autostart_config.enabled
            );
            SlideshowCommand::NoOp
        }

        "cycle_scaling" => SlideshowCommand::CycleScaling,

        "shutdown" => SlideshowCommand::Shutdown,

        _ => return Err(anyhow!("Unknown command: {command}")),
    };

    Ok(command)
}
