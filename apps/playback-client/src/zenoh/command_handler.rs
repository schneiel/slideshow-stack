use crate::slideshow_executor::Command;
use crate::video_player::VideoCommand;
use crate::sync::SyncService;
use anyhow::{Context, Result};
use std::sync::Arc;
use std::sync::mpsc::SyncSender;
use tracing::{debug, warn};
use video_rs::path_utils::is_video_file;

pub struct CommandHandler {
    device_id: String,
    media_directory: String,
    autostart_file: String,
    slideshow_command_tx: SyncSender<Command>,
    video_command_tx: SyncSender<VideoCommand>,
    sync_service: Arc<SyncService>,
}

impl CommandHandler {
    pub const fn new(
        device_id: String,
        media_directory: String,
        autostart_file: String,
        slideshow_command_tx: SyncSender<Command>,
        video_command_tx: SyncSender<VideoCommand>,
        sync_service: Arc<SyncService>,
    ) -> Self {
        Self {
            device_id,
            media_directory,
            autostart_file,
            slideshow_command_tx,
            video_command_tx,
            sync_service,
        }
    }

    #[allow(clippy::too_many_lines)]
    pub async fn handle_command(&self, payload: &[u8]) -> Result<()> {
        let payload_len = payload.len();
        debug!("Raw Zenoh payload: {} bytes", payload_len);

        let msg: crate::zenoh::commands::ZenohCommand =
            serde_json::from_slice(payload).context("Failed to deserialize message")?;

        debug!(
            "Parsed Zenoh command: device_id='{}', our_id='{}', broadcast={}, command='{}'",
            msg.device_id, self.device_id, msg.broadcast, msg.command
        );

        if !msg.broadcast && msg.device_id != self.device_id {
            debug!(
                "Ignoring command for different device: {} (target: {})",
                msg.device_id, self.device_id
            );
            return Ok(());
        }

        tracing::info!(
            "Command accepted: {} (broadcast: {})",
            msg.command,
            msg.broadcast
        );

        if msg.command == "start"
            && let Some(config) = &msg.config
        {
                let media_files: Vec<String> = if config.is_array() {
                    config.as_array()
                        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                        .unwrap_or_default()
                } else if let Some(obj) = config.as_object() {
                    obj.get("media_files")
                        .and_then(|v| v.as_array())
                        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                        .unwrap_or_default()
                } else {
                    Vec::new()
                };

                if !media_files.is_empty() && media_files.iter().all(|f| is_video_file(f)) {
                    debug!("Handling as video command");
                    debug!(
                        "Syncing {} video files before playback...",
                        media_files.len()
                    );
                    let sync_result = self
                        .sync_service
                        .download_missing_media(media_files.clone())
                        .await;
                    if let Err(e) = sync_result {
                        warn!(
                            "Video media sync failed: {}, continuing with available media",
                            e
                        );
                    }
                    if let Some(first_file) = media_files.first() {
                        let video_cmd = VideoCommand::LoadVideo {
                            filename: first_file.clone(),
                        };
                        if let Err(e) = self.video_command_tx.try_send(video_cmd) {
                            warn!("Failed to send video command: {}", e);
                        }
                    }
                    let _ = self.slideshow_command_tx.try_send(Command::StopSlideshow);
                    let _ = self.video_command_tx.try_send(VideoCommand::PlayVideo);
                    return Ok(());
                }
        }

        if msg.command == "start_video"
            && let Some(config) = &msg.config
        {
            if let Some(filename) = config.get("filename").and_then(|v| v.as_str()) {
                debug!("Starting video: {}", filename);
                let video_cmd = VideoCommand::LoadVideo {
                    filename: filename.to_string(),
                };
                if let Err(e) = self.video_command_tx.try_send(video_cmd) {
                    warn!("Failed to send video load command: {}", e);
                }
                let _ = self.slideshow_command_tx.try_send(Command::StopSlideshow);
                let _ = self.video_command_tx.try_send(VideoCommand::PlayVideo);
            }
            return Ok(());
        }

        if msg.command == "stop_video" {
            let video_stop = VideoCommand::StopVideo;
            let _ = self.video_command_tx.try_send(video_stop);
            return Ok(());
        }

        if msg.command == "pause_video" {
            let video_pause = VideoCommand::PauseVideo;
            let _ = self.video_command_tx.try_send(video_pause);
            return Ok(());
        }

        if msg.command == "resume_video" {
            let video_resume = VideoCommand::PlayVideo;
            if let Err(e) = self.video_command_tx.try_send(video_resume) {
                warn!("Failed to send video resume command: {}", e);
            }
            return Ok(());
        }

        if msg.command == "stop_slideshow" {
            let _ = self.slideshow_command_tx.try_send(Command::StopSlideshow);
            return Ok(());
        }

        if msg.command == "pause_slideshow" {
            let _ = self.slideshow_command_tx.try_send(Command::PauseSlideshow);
            return Ok(());
        }

        if msg.command == "resume_slideshow" {
            let _ = self.slideshow_command_tx.try_send(Command::ResumeSlideshow);
            return Ok(());
        }

        if msg.command == "set_video_scaling_mode" {
            let mode = i32::try_from(
                msg.config
                    .as_ref()
                    .and_then(|c| c.get("mode"))
                    .and_then(serde_json::Value::as_i64)
                    .unwrap_or(1),
            )
            .unwrap_or(1);
            let video_mode = video_rs::ScalingMode::from(mode);
            let _ = self.video_command_tx.try_send(VideoCommand::SetScalingMode { mode: video_mode });
            return Ok(());
        }

        if msg.command == "set_slideshow_scaling_mode" {
            let mode = i32::try_from(
                msg.config
                    .as_ref()
                    .and_then(|c| c.get("mode"))
                    .and_then(serde_json::Value::as_i64)
                    .unwrap_or(1),
            )
            .unwrap_or(1);
            let _ = self.slideshow_command_tx.try_send(Command::SetScalingMode { mode });
            return Ok(());
        }

        let command = match crate::zenoh::commands::parse_command_impl(
            &msg.command,
            msg.config,
            &self.media_directory,
            &self.autostart_file,
        ) {
            Ok(cmd) => cmd,
            Err(e) => {
                warn!("Parse error: {}", e);
                return Err(anyhow::anyhow!("Invalid payload for command: Parse error: {e}"));
            }
        };

if let Command::StartSlideshow { ref data, .. } = command {
            debug!(
                "Syncing {} media files before slideshow start...",
                data.media_files.len()
            );
            let _ = self.video_command_tx.try_send(VideoCommand::StopVideo);
            let sync_result = self
                .sync_service
                .download_missing_media(data.media_files.clone())
                .await;
            if let Err(e) = sync_result {
                warn!("Media sync failed: {}", e);
            }
        }

        debug!("Sending slideshow command to main thread via channel");
        if let Err(e) = self.slideshow_command_tx.try_send(command) {
            match e {
                std::sync::mpsc::TrySendError::Full(_) => {
                    tracing::error!("Slideshow command buffer full - main thread too slow, dropping command");
                }
                std::sync::mpsc::TrySendError::Disconnected(_) => {
                    warn!("Main thread disconnected");
                }
            }
            return Ok(());
        }

        Ok(())
    }
}

pub async fn create_subscriptions(
    session: &zenoh::Session,
    device_id: &str,
) -> Result<(
    zenoh::pubsub::Subscriber<zenoh::handlers::FifoChannelHandler<zenoh::sample::Sample>>,
    zenoh::pubsub::Subscriber<zenoh::handlers::FifoChannelHandler<zenoh::sample::Sample>>,
)> {
    use crate::zenoh::keyexpr::{device_command_key, devices_command_key};

    let device_key = device_command_key(device_id);
    let device_sub = session.declare_subscriber(&device_key).await
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    debug!("Listening for device-specific commands on: {}", device_key);

    let broadcast_key = devices_command_key();
    let broadcast_sub = session.declare_subscriber(broadcast_key).await
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    debug!("Listening for broadcast commands on: {}", broadcast_key);

    Ok((device_sub, broadcast_sub))
}