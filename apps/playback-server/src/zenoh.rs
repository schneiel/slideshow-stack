use anyhow::Result;

pub const KEY_DEVICES_COMMAND: &str = "slideshow/devices/command";
pub const KEY_SLIDESHOW_STATE_WILDCARD: &str = "slideshow/state/**";
pub const KEY_VIDEO_STATE_WILDCARD: &str = "slideshow/video/state/**";

/// Connects to Zenoh broker at the given endpoint.
///
/// Uses the default configuration from `DEFAULT_CONFIG.json5` and adds
/// the endpoint as a connect endpoint.
pub async fn connect(endpoint: &str) -> Result<zenoh::Session> {
    let mut config = zenoh::Config::from_json5(include_str!("../DEFAULT_CONFIG.json5"))
        .map_err(|e| anyhow::anyhow!("Failed to load default config: {e}"))?;
    config
        .insert_json5("connect/endpoints", &format!(r#"["{endpoint}"]"#))
        .map_err(|e| anyhow::anyhow!("Failed to set zenoh endpoint: {e}"))?;
    let session = zenoh::open(config).await.map_err(|e| anyhow::anyhow!("{e}"))?;
    Ok(session)
}

/// Connects to Zenoh with automatic retry and exponential backoff.
///
/// Retries connection with exponential backoff from 1000ms to 30000ms.
/// Logs each failed attempt at warn level.
///
/// # Arguments
///
/// * `endpoint` - Zenoh broker endpoint
/// * `max_retries` - Optional maximum number of retry attempts
pub async fn connect_with_retry(
    endpoint: &str,
    max_retries: Option<u32>,
) -> Result<zenoh::Session> {
    let mut attempt = 0u32;
    let mut period_ms: u64 = 1000;
    let period_max_ms: u64 = 30000;
    let increase_factor: u64 = 2;

    loop {
        match connect(endpoint).await {
            Ok(session) => {
                if attempt > 0 {
                    tracing::info!("Successfully reconnected to Zenoh after {} attempts", attempt);
                }
                return Ok(session);
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

/// Publishes a JSON-serializable payload to a Zenoh key expression.
///
/// # Type Parameters
///
/// * `T` - Must implement `serde::Serialize` and `Sync`
///
/// # Arguments
///
/// * `session` - Zenoh session for publishing
/// * `key_expr` - Zenoh key expression to publish to
/// * `payload` - Data to serialize and publish
pub async fn publish_json<T: serde::Serialize + Sync>(
    session: &zenoh::Session,
    key_expr: &str,
    payload: &T,
) -> Result<()> {
    use anyhow::Context;
    let payload_bytes = serde_json::to_vec(payload).context("failed to serialize payload to JSON")?;
    let publisher = session.declare_publisher(key_expr).await
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    publisher.put(payload_bytes).await.map_err(|e| anyhow::anyhow!("{e}"))?;
    Ok(())
}

/// Generates the Zenoh key expression for sending commands to a specific device.
///
/// # Arguments
///
/// * `device_id` - Target device identifier
pub fn command_key_for_device(device_id: &str) -> String {
    format!("slideshow/device/{device_id}/command")
}

pub const fn command_key_broadcast() -> &'static str {
    KEY_DEVICES_COMMAND
}