use video_rs::path_utils::sanitize_and_join_all;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use slideshow_rs::{ScalingMode, Slideshow, SlideshowStateData};
use std::path::{Path, PathBuf};
use tracing::trace;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Command {
    StartSlideshow { data: StartSlideshowData },
    StopSlideshow,
    PauseSlideshow,
    ResumeSlideshow,
    NextImage,
    PreviousImage,
    SetScalingMode { mode: i32 },
    CycleScaling,
    Shutdown,
    NoOp,
}

#[derive(Debug, Clone)]
pub struct StartSlideshowData {
    pub name: String,
    pub interval_seconds: f64,
    pub shuffle_enabled: bool,
    pub loop_enabled: bool,
    pub scaling_mode: ScalingMode,
    pub media_files: Vec<String>,
    pub media_directory: String,
}

impl<'de> Deserialize<'de> for StartSlideshowData {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;

        #[derive(Deserialize)]
        struct RawStartSlideshowData {
            name: String,
            interval_seconds: f64,
            shuffle_enabled: bool,
            loop_enabled: bool,
            #[serde(default)]
            scaling_mode: Option<Value>,
            media_files: Vec<String>,
            #[serde(default)]
            media_directory: String,
        }

        let raw = RawStartSlideshowData::deserialize(deserializer)?;

        let scaling_mode = match raw.scaling_mode {
            Some(Value::String(s)) => match s.to_lowercase().as_str() {
                "none" => ScalingMode::None,
                "fill" => ScalingMode::FillToScreen,
                "stretch" => ScalingMode::StretchToFit,
                _ => ScalingMode::FitToScreen,
            },
            Some(Value::Number(n)) => {
                let n_i64 = n
                    .as_i64()
                    .ok_or_else(|| D::Error::custom("scaling_mode is not a valid i64"))?;
                let n_i32 = i32::try_from(n_i64)
                    .map_err(|_| D::Error::custom("scaling_mode value out of range"))?;
                ScalingMode::from(n_i32)
            }
            _ => ScalingMode::FitToScreen,
        };

        Ok(Self {
            name: raw.name,
            interval_seconds: raw.interval_seconds,
            shuffle_enabled: raw.shuffle_enabled,
            loop_enabled: raw.loop_enabled,
            scaling_mode,
            media_files: raw.media_files,
            media_directory: raw.media_directory,
        })
    }
}

impl Serialize for StartSlideshowData {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("StartSlideshowData", 7)?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("interval_seconds", &self.interval_seconds)?;
        state.serialize_field("shuffle_enabled", &self.shuffle_enabled)?;
        state.serialize_field("loop_enabled", &self.loop_enabled)?;
        state.serialize_field("scaling_mode", self.scaling_mode.as_str())?;
        state.serialize_field("media_files", &self.media_files)?;
        state.serialize_field("media_directory", &self.media_directory)?;
        state.end()
    }
}

pub struct Executor<'a, B: sdl3_rs::Renderer> {
    slideshow: Slideshow<'a, B>,
    media_directory: String,
}

fn paths_to_strings(paths: Vec<PathBuf>) -> Vec<String> {
    paths
        .into_iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect()
}

impl<'a, B: sdl3_rs::Renderer> Executor<'a, B> {
    pub const fn new(slideshow: Slideshow<'a, B>, media_directory: String) -> Self {
        Self {
            slideshow,
            media_directory,
        }
    }

    #[allow(clippy::needless_pass_by_ref_mut)]
    pub fn execute_command(&mut self, command: &Command) -> Result<()> {
        trace!("Executing command: {:?}", command);

        match command {
            Command::StartSlideshow { data } => {
                let media_dir = Path::new(&self.media_directory);
                let sanitized_paths = sanitize_and_join_all(media_dir, &data.media_files)
                    .map_err(|e| anyhow::anyhow!("Invalid media file path: {e}"))?;

                let path_strings = paths_to_strings(sanitized_paths);

                self.slideshow.push_start_command(
                    data.name.clone(),
                    data.interval_seconds,
                    data.shuffle_enabled,
                    data.loop_enabled,
                    path_strings,
                )?;

                self.slideshow.push_scaling_mode_command(data.scaling_mode)?;
            }

            Command::StopSlideshow => {
                self.slideshow.push_command(slideshow_rs::command::Command::Stop)?;
            }

            Command::PauseSlideshow => {
                self.slideshow.push_pause_command(true)?;
            }

            Command::ResumeSlideshow => {
                self.slideshow.push_pause_command(false)?;
            }

            Command::NextImage => {
                self.slideshow.push_command(slideshow_rs::command::Command::Next)?;
            }

            Command::PreviousImage => {
                self.slideshow.push_command(slideshow_rs::command::Command::Previous)?;
            }

            Command::SetScalingMode { mode } => {
                let scaling_mode = ScalingMode::from(*mode);
                tracing::debug!(target: "scaling", "SetScalingMode received: raw={}, ScalingMode::{:?}", *mode, scaling_mode);
                self.slideshow.push_scaling_mode_command(scaling_mode)?;
                tracing::debug!(target: "scaling", "push_scaling_mode_command completed");
            }

            Command::CycleScaling => {
                let current_mode = self.slideshow.get_scaling_mode();
                let next_mode = current_mode.next();
                tracing::debug!(target: "scaling", "CycleScaling: {:?} -> {:?}", current_mode, next_mode);
                self.slideshow.push_scaling_mode_command(next_mode)?;
            }

            Command::Shutdown => {
                self.slideshow.push_command(slideshow_rs::command::Command::Shutdown)?;
            }

            Command::NoOp => {}
        }

        Ok(())
    }

    #[allow(clippy::unnecessary_wraps)]
    pub fn tick_slideshow(&mut self, delta_time: Option<f64>) -> Result<()> {
        let dt = delta_time.unwrap_or(1.0 / 60.0);
        self.slideshow.tick(dt);
        Ok(())
    }

    #[allow(dead_code, clippy::unnecessary_wraps)]
    pub fn render_slideshow(&mut self) -> Result<()> {
        self.slideshow.render();
        Ok(())
    }

    #[allow(clippy::needless_pass_by_ref_mut)]
    pub fn shutdown(&mut self) -> Result<()> {
        self.slideshow.push_command(slideshow_rs::command::Command::Shutdown)?;
        Ok(())
    }

    #[allow(clippy::unnecessary_wraps)]
    pub fn poll_state(&self) -> Option<SlideshowStateData> {
        Some(self.slideshow.get_state())
    }

    pub fn get_state_revision(&self) -> u32 {
        self.slideshow.get_state_generation()
    }
}
