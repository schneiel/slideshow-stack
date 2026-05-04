use crate::zenoh::ConnectionState;
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::watch;

pub async fn create_zenoh_connection(
    endpoint: &str,
) -> Result<(
    zenoh::Session,
    Arc<watch::Sender<bool>>,
    watch::Receiver<bool>,
    Arc<parking_lot::Mutex<ConnectionState>>,
)> {
    let connection_state: Arc<parking_lot::Mutex<ConnectionState>> =
        Arc::new(parking_lot::Mutex::new(ConnectionState::Connecting));
    let (reconnect_notify, reconnect_rx) = watch::channel(false);

    let reconnect_notify = Arc::new(reconnect_notify);

    let mut config = zenoh::Config::from_json5(include_str!("../../DEFAULT_CONFIG.json5"))
        .map_err(|e| anyhow::anyhow!("Failed to load default config: {e}"))?;
    config
        .insert_json5("connect/endpoints", &format!(r#"["{endpoint}"]"#))
        .map_err(|e| anyhow::anyhow!("Failed to set zenoh endpoint: {e}"))?;
    let session = zenoh::open(config).await.map_err(|e| anyhow::anyhow!("{e}"))?;

    *connection_state.lock() = ConnectionState::Connected;
    let _ = reconnect_notify.send(true);

    Ok((session, reconnect_notify, reconnect_rx, connection_state))
}

pub async fn create_zenoh_connection_with_retry(
    endpoint: &str,
    max_retries: Option<u32>,
) -> Result<(
    zenoh::Session,
    Arc<watch::Sender<bool>>,
    watch::Receiver<bool>,
    Arc<parking_lot::Mutex<ConnectionState>>,
)> {
    let mut attempt = 0u32;
    let mut period_ms: u64 = 1000;
    let period_max_ms: u64 = 30000;
    let increase_factor: u64 = 2;

    loop {
        match create_zenoh_connection(endpoint).await {
            Ok(result) => {
                if attempt > 0 {
                    tracing::info!("Successfully reconnected to Zenoh after {} attempts", attempt);
                }
                return Ok(result);
            }
            Err(e) => {
                attempt += 1;
                let max_retries_msg = max_retries.map(|m| format!("{m}/")).unwrap_or_default();
                tracing::warn!(
                    "Failed to connect to Zenoh (attempt {}{}): {}. Retrying in {}ms...",
                    max_retries_msg,
                    attempt,
                    e,
                    period_ms
                );

                if let Some(max) = max_retries && attempt >= max {
                    return Err(anyhow::anyhow!(
                        "Failed to connect to Zenoh after {attempt} attempts: {e}"
                    ));
                }

                tokio::time::sleep(tokio::time::Duration::from_millis(period_ms)).await;
                period_ms = (period_ms * increase_factor).min(period_max_ms);
            }
        }
    }
}

