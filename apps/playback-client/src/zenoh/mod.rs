use std::sync::Arc;
use anyhow::Result;
use tracing::{debug, info, warn, error};

use crate::config::{ClientState, Config, save_client_state};
use crate::slideshow_executor::Command;
use crate::video_player::VideoCommand;
use crate::sync::SyncService;

pub mod command_handler;
pub mod commands;
pub mod connection;
pub mod keyexpr;

pub use commands::{SlideshowState, VideoState};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionState {
    Connecting,
    Connected,
    Disconnected,
}

/// Manages Zenoh connections and state handling for the playback client.
///
/// Responsible for:
/// - Establishing and maintaining Zenoh session
/// - Publishing connect/disconnect lifecycle
/// - Subscribing to device-specific and broadcast commands
/// - Publishing slideshow and video state updates to the server
///
/// # Thread Safety
///
/// This type is `Send + Sync` as it uses thread-safe primitives internally.
pub struct ZenohManager {
    session: zenoh::Session,
    device_id: String,
    display_name: String,
    config: Config,
    slideshow_command_tx: std::sync::mpsc::SyncSender<Command>,
    video_command_tx: std::sync::mpsc::SyncSender<VideoCommand>,
    sync_service: Arc<SyncService>,
    connection_state: Arc<parking_lot::Mutex<ConnectionState>>,
    pub local_slideshow_state: Arc<parking_lot::Mutex<SlideshowState>>,
    pub local_video_state: Arc<parking_lot::Mutex<VideoState>>,
    last_published_slideshow: Arc<parking_lot::Mutex<SlideshowState>>,
    last_published_video: Arc<parking_lot::Mutex<VideoState>>,
    reconnect_rx: tokio::sync::watch::Receiver<bool>,
    tick_rx: tokio::sync::mpsc::Receiver<()>,
}

impl ZenohManager {
    /// Creates a new `ZenohManager` and establishes a connection.
    ///
    /// Loads existing client state or creates new identity.
    /// Implements exponential backoff retry on connection failure.
    ///
    /// # Arguments
    ///
    /// * `cfg` - Client configuration
    /// * `slideshow_command_tx` - Channel for sending slideshow commands to the executor
    /// * `video_command_tx` - Channel for sending video commands to the player
    /// * `slideshow_state_rx` - Channel for receiving slideshow state updates
    /// * `video_state_rx` - Channel for receiving video state updates
    /// * `sync_service` - Service for syncing media files
    pub async fn new(
        cfg: Config,
        slideshow_command_tx: std::sync::mpsc::SyncSender<Command>,
        video_command_tx: std::sync::mpsc::SyncSender<VideoCommand>,
        sync_service: Arc<SyncService>,
        slideshow_state: Arc<parking_lot::Mutex<SlideshowState>>,
        video_state: Arc<parking_lot::Mutex<VideoState>>,
    ) -> Result<(Self, tokio::sync::mpsc::Sender<()>)> {
        info!("Connecting to Zenoh at {}", cfg.zenoh_endpoint);

        let (tick_tx, tick_rx) = tokio::sync::mpsc::channel(1);

        let existing_state = crate::config::load_or_create_client_state(&cfg.client_state_file)?;
        let display_name = existing_state.display_name.clone();
        let device_id = existing_state.device_id.clone();

        let (session, _reconnect_notify, reconnect_rx, connection_state) =
            connection::create_zenoh_connection_with_retry(&cfg.zenoh_endpoint, None).await?;

        let updated_state = ClientState {
            device_id: device_id.clone(),
            display_name: display_name.clone(),
            last_connected: chrono::Utc::now().timestamp(),
        };
        save_client_state(&cfg.client_state_file, &updated_state)?;

        Ok((Self {
            session,
            device_id,
            display_name,
            config: cfg,
            slideshow_command_tx,
            video_command_tx,
            sync_service,
            connection_state,
            local_slideshow_state: slideshow_state,
            local_video_state: video_state,
            last_published_slideshow: Arc::new(parking_lot::Mutex::new(SlideshowState::default())),
            last_published_video: Arc::new(parking_lot::Mutex::new(VideoState::default())),
            reconnect_rx,
            tick_rx,
        }, tick_tx))
    }

    async fn publish_current_state(&self) {
        let state = self.local_slideshow_state.lock().clone();
        if let Err(e) = self.publish_slideshow_state(&state).await {
            debug!("Failed to publish slideshow state: {}", e);
        }
        let video_state = self.local_video_state.lock().clone();
        if let Err(e) = self.publish_video_state(&video_state).await {
            debug!("Failed to publish video state: {}", e);
        }
    }

    async fn publish_slideshow_state(&self, state: &SlideshowState) -> Result<()> {
        use keyexpr::device_state_key;
        let key = device_state_key(&self.device_id);
        let payload_json = serde_json::json!({
            "device_id": self.device_id,
            "display_name": self.display_name,
            "state": state,
        });
        let payload = serde_json::to_vec(&payload_json)?;
        let publisher = self.session.declare_publisher(&key).await
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        publisher.put(payload).await.map_err(|e| anyhow::anyhow!("{e}"))?;
        Ok(())
    }

    async fn publish_video_state(&self, state: &VideoState) -> Result<()> {
        use keyexpr::device_video_state_key;
        let key = device_video_state_key(&self.device_id);
        let payload_json = serde_json::json!({
            "device_id": self.device_id,
            "display_name": self.display_name,
            "state": state,
        });
        let payload = serde_json::to_vec(&payload_json)?;
        let publisher = self.session.declare_publisher(&key).await
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        publisher.put(payload).await.map_err(|e| anyhow::anyhow!("{e}"))?;
        Ok(())
    }

    /// Runs the Zenoh subscription loop until shutdown is received.
    ///
    /// Handles:
    /// - Device-specific commands
    /// - Broadcast commands
    /// - Slideshow state updates
    /// - Automatic reconnection on session loss
    ///
    /// # Arguments
    ///
    /// * `shutdown_rx` - Channel for receiving shutdown signal
    #[allow(clippy::too_many_lines)]
    pub async fn run(mut self, mut shutdown_rx: tokio::sync::mpsc::Receiver<()>) -> Result<()> {
        info!("Zenoh subscription loop started");

        {
            let is_connected = *self.connection_state.lock() == ConnectionState::Connected;
            if !is_connected {
                self.reconnect_rx.changed().await?;
            }
        }

        let command_handler = command_handler::CommandHandler::new(
            self.device_id.clone(),
            self.config.media_directory.clone(),
            self.config.autostart_file.clone(),
            self.slideshow_command_tx.clone(),
            self.video_command_tx.clone(),
            Arc::clone(&self.sync_service),
        );

        self.publish_current_state().await;

        let mut session_alive = true;
        let mut retry_period_ms: u64 = 1000;
        let retry_period_max: u64 = 30000;
        let retry_increase_factor: u64 = 2;

        let mut device_sub: Option<zenoh::pubsub::Subscriber<zenoh::handlers::FifoChannelHandler<zenoh::sample::Sample>>> = None;
        let mut broadcast_sub: Option<zenoh::pubsub::Subscriber<zenoh::handlers::FifoChannelHandler<zenoh::sample::Sample>>> = None;

        loop {
            if !session_alive {
                warn!("Session lost, attempting to reconnect...");
                *self.connection_state.lock() = ConnectionState::Disconnected;

                match connection::create_zenoh_connection_with_retry(&self.config.zenoh_endpoint, None).await {
                    Ok((session, _, reconnect_rx, connection_state)) => {
                        self.session = session;
                        self.reconnect_rx = reconnect_rx;
                        self.connection_state = connection_state;
                        *self.connection_state.lock() = ConnectionState::Connected;
                        retry_period_ms = 1000;
                        info!("Successfully reconnected to Zenoh");

                        device_sub = None;
                        broadcast_sub = None;
                    }
                    Err(e) => {
                        error!("Failed to reconnect to Zenoh: {}, retrying in {}ms", e, retry_period_ms);
                        tokio::time::sleep(tokio::time::Duration::from_millis(retry_period_ms)).await;
                        retry_period_ms = (retry_period_ms * retry_increase_factor).min(retry_period_max);
                        continue;
                    }
                }
            }

            if device_sub.is_none() {
                match command_handler::create_subscriptions(&self.session, &self.device_id).await {
                    Ok((d, b)) => {
                        device_sub = Some(d);
                        broadcast_sub = Some(b);
                        session_alive = true;
                        self.publish_current_state().await;
                    }
                    Err(e) => {
                        error!("Failed to create subscriptions: {}, session may be dead", e);
                        session_alive = false;
                        continue;
                    }
                }
            }

            let device_sub = device_sub.as_ref().unwrap();
            let broadcast_sub = broadcast_sub.as_ref().unwrap();

            tokio::select! {
                Some(()) = shutdown_rx.recv() => {
                    info!("Shutdown signal received");
                    if let Err(e) = self.slideshow_command_tx.try_send(Command::StopSlideshow) {
                        warn!("Failed to send stop slideshow command on shutdown: {}", e);
                    }
                    debug!("Zenoh manager shutting down");
                    return Ok(());
                }

                device_result = device_sub.recv_async() => {
                    match device_result {
                        Ok(sample) => {
                            let payload = sample.payload().to_bytes();
                            if let Err(e) = command_handler.handle_command(&payload).await {
                                warn!("Failed to handle command: {}", e);
                            }
                        }
                        Err(e) => {
                            warn!("Device subscription error: {}, attempting to recover", e);
                            session_alive = false;
                        }
                    }
                }

                broadcast_result = broadcast_sub.recv_async() => {
                    match broadcast_result {
                        Ok(sample) => {
                            let payload = sample.payload().to_bytes();
                            if let Err(e) = command_handler.handle_command(&payload).await {
                                warn!("Failed to handle broadcast command: {}", e);
                            }
                        }
                        Err(e) => {
                            warn!("Broadcast subscription error: {}, attempting to recover", e);
                            session_alive = false;
                        }
                    }
                }

                Some(()) = self.tick_rx.recv() => {
                    let (current_slideshow, current_video) = {
                        let slideshow = self.local_slideshow_state.lock().clone();
                        let video = self.local_video_state.lock().clone();
                        (slideshow, video)
                    };

                    let slideshow_changed = current_slideshow != *self.last_published_slideshow.lock();
                    if slideshow_changed {
                        if let Err(e) = self.publish_slideshow_state(&current_slideshow).await {
                            debug!("Failed to publish slideshow state: {}", e);
                        } else {
                            *self.last_published_slideshow.lock() = current_slideshow;
                        }
                    }

                    let video_changed = current_video != *self.last_published_video.lock();
                    if video_changed {
                        if let Err(e) = self.publish_video_state(&current_video).await {
                            debug!("Failed to publish video state: {}", e);
                        } else {
                            *self.last_published_video.lock() = current_video;
                        }
                    }
                }
            }
        }
    }
}