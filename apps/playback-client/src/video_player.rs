use video_rs::{Command as VideoCommandType, ScalingMode, VideoPlayer as VideoPlayerCore, VideoStateData};
use video_rs::path_utils::sanitize_and_join_all;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use sdl3_rs::Renderer;
use std::path::Path;
use tracing::{info, trace};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VideoCommand {
    LoadVideo { filename: String },
    PlayVideo,
    PauseVideo,
    StopVideo,
    SetScalingMode { mode: ScalingMode },
    NoOp,
}

pub struct VideoPlayer<'a, R: Renderer> {
    video: VideoPlayerCore<'a, R>,
    media_directory: String,
}

impl<'a, R: Renderer> VideoPlayer<'a, R> {
    pub fn new(renderer: &'a R, media_directory: String) -> Self {
        Self {
            video: VideoPlayerCore::new(renderer),
            media_directory,
        }
    }

    pub fn execute_command(&mut self, command: &VideoCommand) -> Result<()> {
        trace!("Executing video command: {:?}", command);

        match command {
            VideoCommand::LoadVideo {
                filename,
            } => {
                let media_dir = Path::new(&self.media_directory);
                let sanitized_path = sanitize_and_join_all(media_dir, std::slice::from_ref(filename))
                    .map_err(|e| anyhow::anyhow!("Invalid media file path: {e}"))?;

                let path_str = sanitized_path
                    .first()
                    .ok_or_else(|| anyhow::anyhow!("No path returned"))?
                    .to_string_lossy()
                    .to_string();

                self.video.load(&path_str)?;
                info!("Loaded video: {}", path_str);
            }

            VideoCommand::PlayVideo => {
                self.video.push_command(VideoCommandType::Resume);
            }

            VideoCommand::PauseVideo => {
                self.video.push_command(VideoCommandType::Pause);
            }

            VideoCommand::StopVideo => {
                self.video.push_command(VideoCommandType::Stop);
            }

            VideoCommand::SetScalingMode { mode } => {
                self.video.push_command(VideoCommandType::SetScalingMode(*mode));
            }

            VideoCommand::NoOp => {}
        }

        Ok(())
    }

    pub fn tick(&mut self, delta_time: f64) -> Result<()> {
        self.video.tick(delta_time)?;
        Ok(())
    }

    #[allow(clippy::unnecessary_wraps)]
    pub fn render(&self) -> Result<()> {
        self.video.render();
        Ok(())
    }

    pub fn is_active(&self) -> bool {
        let active = self.video.has_player();
        tracing::trace!(target: "video", "is_active() = {}", active);
        active
    }

    pub fn get_state_revision(&self) -> u32 {
        self.video.get_state_generation()
    }

    #[allow(clippy::unnecessary_wraps)]
    pub fn poll_state(&self) -> Option<VideoStateData> {
        Some(self.video.get_state())
    }

    pub fn shutdown(&mut self) {
        self.video.push_command(VideoCommandType::Stop);
    }
}
