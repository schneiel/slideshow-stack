use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::env;
use std::path::PathBuf;
use tracing::{debug, info};

/// Persistent client state for connection tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientState {
    pub device_id: String,
    pub display_name: String,
    pub last_connected: i64,
}

/// Configuration for the playback client.
///
/// Loaded from environment variables with sensible defaults.
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    /// Zenoh connection endpoint (e.g., "udp/127.0.0.1:7447")
    pub zenoh_endpoint: String,
    /// Local directory containing media files
    pub media_directory: String,
    /// HTTP URL of the media sync server
    pub sync_server_url: String,
    /// Path to autostart configuration file
    pub autostart_file: String,
    /// Path to persistent client state file (not serialized)
    #[serde(skip)]
    pub client_state_file: PathBuf,
    /// Target FPS for the main display loop (default: 30 for low-core systems)
    #[serde(skip)]
    pub target_fps: i32,
    /// Log level filter (default: "info")
    #[serde(skip)]
    pub log_level: String,
    /// Override display width (default: auto-detect)
    #[serde(skip)]
    pub display_width: Option<i32>,
    /// Override display height (default: auto-detect)
    #[serde(skip)]
    pub display_height: Option<i32>,
}

impl Config {
    /// Logs the current configuration at debug level.
    pub fn print(&self) {
        info!("=== Playback Client Configuration ===");
        info!("Zenoh Endpoint: {}", self.zenoh_endpoint);
        info!("Media Directory: {}", self.media_directory);
        info!("Sync Server: {}", self.sync_server_url);
        info!("Autostart File: {}", self.autostart_file);
        info!("Client State File: {:?}", self.client_state_file);
        info!("Target FPS: {}", self.target_fps);
        info!("Log Level: {}", self.log_level);
        info!(
            "Display Override: {}x{}",
            self.display_width.map_or_else(|| "auto".to_string(), |v| v.to_string()),
            self.display_height.map_or_else(|| "auto".to_string(), |v| v.to_string())
        );
        info!("=====================================");
    }
}

/// Load or create client state
/// If no state file exists, creates a new one with a unique `device_id` and default display name
pub fn load_or_create_client_state(path: &PathBuf) -> Result<ClientState> {
    if path.exists() {
        let content = std::fs::read_to_string(path).context("Failed to read client state file")?;
        let state: ClientState =
            serde_json::from_str(&content).context("Failed to parse client state file")?;

        debug!(
            "Loaded existing client state: device_id={}, display_name='{}'",
            state.device_id, state.display_name
        );
        Ok(state)
    } else {
        let device_id = uuid::Uuid::new_v4().to_string();
        let display_name = "slideshow-client".to_string();
        let state = ClientState {
            device_id,
            display_name,
            last_connected: chrono::Utc::now().timestamp(),
        };
        save_client_state(path, &state)?;
        debug!(
            "Created new client state: device_id={}, display_name='{}'",
            state.device_id, state.display_name
        );
        Ok(state)
    }
}

/// Saves client state to a JSON file.
///
/// # Arguments
///
/// * `path` - Path to the state file
/// * `state` - Client state to persist
pub fn save_client_state(path: &PathBuf, state: &ClientState) -> Result<()> {
    let content =
        serde_json::to_string_pretty(state).context("Failed to serialize client state")?;

    std::fs::write(path, content).context("Failed to write client state file")?;

    debug!(
        "Saved client state: device_id={}, display_name='{}'",
        state.device_id, state.display_name
    );

    Ok(())
}

/// Loads configuration from environment variables.
///
/// # Environment Variables
///
/// - `ZENOH_CONNECT_ENDPOINTS` (default: "udp/127.0.0.1:7447")
/// - `MEDIA_DIR` (default: "./media")
/// - `SYNC_SERVER_URL` (default: <http://localhost:61532>)
/// - `AUTOSTART_FILE` (default: "./autostart.json")
/// - `CLIENT_STATE_FILE` (default: `./client_state.json`)
///
/// # Errors
///
/// Returns error if required configuration is missing or invalid.
pub fn load_config() -> Result<Config> {
    let zenoh_endpoint =
        env::var("ZENOH_CONNECT_ENDPOINTS").unwrap_or_else(|_| "udp/127.0.0.1:7447".to_string());

    let media_directory = env::var("MEDIA_DIR").unwrap_or_else(|_| "./media".to_string());

    let sync_server_url =
        env::var("SYNC_SERVER_URL").unwrap_or_else(|_| "http://localhost:61532".to_string());

    let autostart_file =
        env::var("AUTOSTART_FILE").unwrap_or_else(|_| "./autostart.json".to_string());

    let client_state_file =
        env::var("CLIENT_STATE_FILE").unwrap_or_else(|_| "./client_state.json".to_string());

    let target_fps: i32 = env::var("TARGET_FPS")
        .unwrap_or_else(|_| "30".to_string())
        .parse()
        .unwrap_or(30);

    let log_level = env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());

    let display_width = env::var("DISPLAY_WIDTH")
        .ok()
        .and_then(|v| v.parse().ok());
    let display_height = env::var("DISPLAY_HEIGHT")
        .ok()
        .and_then(|v| v.parse().ok());

    let cfg = Config {
        zenoh_endpoint,
        media_directory,
        sync_server_url,
        autostart_file,
        client_state_file: PathBuf::from(client_state_file),
        target_fps,
        log_level,
        display_width,
        display_height,
    };

    if cfg.zenoh_endpoint.is_empty() {
        bail!("ZENOH_CONNECT_ENDPOINTS cannot be empty");
    }
    if cfg.media_directory.is_empty() {
        bail!("MEDIA_DIR cannot be empty");
    }

    Ok(cfg)
}
