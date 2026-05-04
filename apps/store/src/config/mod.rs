//! Configuration Module
//!
//! Environment-based configuration management.
//! Loads settings from environment variables with defaults.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Application configuration.
///
/// Loaded from environment variables with sensible defaults.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Server port (default: 61532)
    pub port: u16,
    /// Database URL (default: "sqlite:slideshows.db")
    pub database_url: String,
    /// Media directory path (default: "./media")
    pub media_dir: PathBuf,
    /// Log level (default: "info")
    pub log_level: String,
    /// Enable CORS (default: false)
    pub cors_enabled: bool,
    /// CORS origin URL (required when `cors_enabled` is true)
    pub cors_origin: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            port: 61532,
            database_url: "sqlite:slideshows.db".into(),
            media_dir: PathBuf::from("./media"),
            log_level: "info".into(),
            cors_enabled: false,
            cors_origin: None,
        }
    }
}

/// Loads configuration from environment variables.
///
/// # Errors
///
/// Returns an error if `CORS_ENABLED` is true but `CORS_ORIGIN` is not set
///
/// # Environment Variables
/// - `PORT` - Server port (default: 61532)
/// - `DATABASE_URL` - Database URL (default: "sqlite:slideshows.db")
/// - `MEDIA_DIR` - Media directory (default: "./media")
/// - `RUST_LOG` - Log level (default: "info")
/// - `CORS_ENABLED` - Enable CORS (default: false)
/// - `CORS_ORIGIN` - CORS origin URL (required when `CORS_ENABLED` is true)
///
/// # Example
///
/// ```no_run
/// use store::config::load_config;
///
/// fn main() {
///     let config = load_config().expect("failed to load config");
///     println!("Server will listen on port {}", config.port);
/// }
/// ```
pub fn load_config() -> anyhow::Result<Config> {
    if let Err(e) = dotenvy::dotenv() {
        tracing::debug!("dotenv not loaded (this is normal in production): {}", e);
    }

    let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(61532);

    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:slideshows.db".into());

    let media_dir =
        std::env::var("MEDIA_DIR").map_or_else(|_| PathBuf::from("./media"), PathBuf::from);

    let log_level = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into());

    let cors_enabled = std::env::var("CORS_ENABLED")
        .unwrap_or_else(|_| "false".into())
        .parse()
        .unwrap_or(false);

    let cors_origin = if cors_enabled {
        Some(std::env::var("CORS_ORIGIN").map_err(|e| {
            anyhow::anyhow!("CORS_ORIGIN must be set when CORS_ENABLED is true: {e}")
        })?)
    } else {
        None
    };

    Ok(Config {
        port,
        database_url,
        media_dir,
        log_level,
        cors_enabled,
        cors_origin,
    })
}
