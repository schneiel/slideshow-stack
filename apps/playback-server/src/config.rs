use anyhow::{Context, Result};
use axum::http::HeaderValue;
use std::env;

/// Server configuration loaded from environment variables.
#[derive(Debug, Clone)]
pub struct Config {
    pub http_port: u16,
    pub zenoh_endpoint: String,
    pub cors_enabled: bool,
    pub cors_origin: Option<HeaderValue>,
    pub log_level: String,
}

impl Config {
    /// Load configuration from environment variables with defaults.
    ///
    /// # Environment Variables
    ///
    /// * `HTTP_PORT` - HTTP server port (default: 9247)
    /// * `ZENOH_CONNECT_ENDPOINTS` - Zenoh endpoint (default: "udp/127.0.0.1:7447")
    /// * `RUST_LOG` - Log level filter (default: "info")
    /// * `CORS_ENABLED` - Enable CORS for control-panel access (default: false)
    /// * `CORS_ORIGIN` - Allowed CORS origin (required if `CORS_ENABLED=true`, e.g. <http://localhost:7381>)
    ///
    /// # Returns
    ///
    /// Configuration with validated values
    ///
    /// # Errors
    ///
    /// Returns error if environment variables are invalid
    pub fn from_env() -> Result<Self> {
        let http_port = env::var("HTTP_PORT")
            .unwrap_or_else(|_| "9247".to_string())
            .parse::<u16>()
            .context("HTTP_PORT must be a valid u16")?;

        let zenoh_endpoint = env::var("ZENOH_CONNECT_ENDPOINTS")
            .unwrap_or_else(|_| "udp/127.0.0.1:7447".to_string());

        let cors_enabled = env::var("CORS_ENABLED")
            .unwrap_or_else(|_| "false".to_string())
            .parse::<bool>()
            .context("CORS_ENABLED must be a valid boolean (true/false)")?;

        let cors_origin = if cors_enabled {
            let origin = env::var("CORS_ORIGIN")
                .context("CORS_ORIGIN must be set when CORS_ENABLED is true")?;
            Some(
                origin
                    .parse::<HeaderValue>()
                    .with_context(|| format!("invalid CORS origin: '{origin}'"))?,
            )
        } else {
            None
        };

        let log_level = env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());

        Ok(Self {
            http_port,
            zenoh_endpoint,
            cors_enabled,
            cors_origin,
            log_level,
        })
    }

    /// Get HTTP bind address for server.
    pub fn http_bind_address(&self) -> String {
        format!("0.0.0.0:{}", self.http_port)
    }
}
