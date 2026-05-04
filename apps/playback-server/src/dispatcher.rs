use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use tracing::{debug, warn};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZenohCommand {
    #[serde(rename = "device_id")]
    pub device_id: String,
    #[serde(rename = "timestamp")]
    pub timestamp: i64,
    #[serde(rename = "command")]
    pub command: String,
    #[serde(rename = "config", skip_serializing_if = "Option::is_none")]
    pub config: Option<JsonValue>,
    #[serde(rename = "broadcast")]
    #[serde(default)]
    pub broadcast: bool,
}

impl ZenohCommand {
    pub fn builder() -> ZenohCommandBuilder {
        ZenohCommandBuilder::default()
    }
}

#[derive(Default)]
pub struct ZenohCommandBuilder {
    device_id: Option<String>,
    command: Option<String>,
    config: Option<JsonValue>,
    broadcast: bool,
}

impl ZenohCommandBuilder {
    pub const fn broadcast(mut self) -> Self {
        self.broadcast = true;
        self
    }

    pub fn command(mut self, cmd: impl Into<String>) -> Self {
        self.command = Some(cmd.into());
        self
    }

    pub fn config_opt(mut self, config: Option<JsonValue>) -> Self {
        self.config = config;
        self
    }

    pub fn try_build(self) -> Result<ZenohCommand> {
        let command = self.command.ok_or_else(|| anyhow!("command is required"))?;
        let timestamp = CommandDispatcher::now_timestamp()
            .context("failed to generate timestamp")?;

        Ok(ZenohCommand {
            device_id: self.device_id.unwrap_or_default(),
            timestamp,
            command,
            config: self.config,
            broadcast: self.broadcast,
        })
    }
}

/// Dispatches commands to devices via Zenoh.
///
/// Thread-safe wrapper around a Zenoh session for publishing command messages
/// to connected playback clients.
///
/// # Thread Safety
///
/// This type is `Send + Sync` as it only contains an `Arc<Session>`.
pub struct CommandDispatcher {
    session: Arc<zenoh::Session>,
}

impl CommandDispatcher {
    /// Creates a new `CommandDispatcher` wrapping the given Zenoh session.
    pub const fn new(session: Arc<zenoh::Session>) -> Self {
        Self { session }
    }

    /// Dispatches a command to one or all devices.
    ///
    /// If `command.broadcast` is true, sends to all connected devices.
    /// Otherwise, sends only to the `device_id` specified in the command.
    ///
    /// # Arguments
    ///
    /// * `command` - The command to dispatch
    /// * `all_device_ids` - List of all known connected device IDs
    pub async fn dispatch(&self, command: ZenohCommand, all_device_ids: Vec<String>) {
        if command.broadcast {
            debug!(
                "Broadcasting command '{}' to {} devices",
                command.command,
                all_device_ids.len()
            );
            self.broadcast_command(&command).await;
        } else {
            let device_id = &command.device_id;
            if all_device_ids.contains(device_id) {
                debug!(
                    "Sending command '{}' to device {}",
                    command.command, device_id
                );
                self.send_to_device(device_id, &command).await;
            } else {
                warn!("Device {} not found (not connected)", device_id);
            }
        }
    }

    /// Dispatches a command to multiple target devices with the same config.
    ///
    /// Useful for sending the same command (e.g., "start", "stop") to multiple
    /// specific devices at once.
    ///
    /// # Arguments
    ///
    /// * `command` - Command name (e.g., "start", "stop")
    /// * `target_device_ids` - List of target device IDs
    /// * `config` - Optional configuration payload
    pub async fn dispatch_batch(
        &self,
        command: &str,
        target_device_ids: Vec<String>,
        config: Option<serde_json::Value>,
    ) {
        if target_device_ids.is_empty() {
            warn!("No target devices provided for command '{}'", command);
            return;
        }

        debug!(
            "Dispatching command '{}' to {} devices",
            command,
            target_device_ids.len()
        );

        let base_timestamp = match Self::now_timestamp() {
            Ok(ts) => ts,
            Err(e) => {
                warn!("Failed to generate timestamp for dispatch_batch: {}", e);
                return;
            }
        };
        let command_str = command.to_string();

        let final_config = config.unwrap_or(serde_json::Value::Null);

        for device_id in target_device_ids {
            let zenoh_command = ZenohCommand {
                device_id: device_id.clone(),
                timestamp: base_timestamp,
                command: command_str.clone(),
                config: Some(final_config.clone()),
                broadcast: false,
            };

            self.send_to_device(&device_id, &zenoh_command).await;
        }
    }

    async fn broadcast_command(&self, command: &ZenohCommand) {
        let key_expr = crate::zenoh::command_key_broadcast();
        debug!("Publishing broadcast to key expression: {}", key_expr);
        match crate::zenoh::publish_json(&self.session, key_expr, command).await {
            Ok(()) => {}
            Err(e) => tracing::error!(
                "Failed to publish broadcast command '{}': {}",
                command.command,
                e
            ),
        }
    }

    async fn send_to_device(&self, device_id: &str, command: &ZenohCommand) {
        let key_expr = crate::zenoh::command_key_for_device(device_id);
        debug!("Publishing to key expression: {}", key_expr);
        match crate::zenoh::publish_json(&self.session, &key_expr, command).await {
            Ok(()) => {
                debug!(
                    "Command '{}' dispatched to device {}",
                    command.command, device_id
                );
            }
            Err(e) => tracing::error!(
                "Failed to publish command '{}' for device {}: {}",
                command.command,
                device_id,
                e
            ),
        }
    }

    /// Returns the current timestamp as nanoseconds since UNIX epoch.
    ///
    /// # Errors
    ///
    /// Returns an error if the system time is before UNIX epoch or overflows i64.
    pub fn now_timestamp() -> Result<i64> {
        use std::time::SystemTime;

        let dur = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map_err(|e| anyhow!("system time is before UNIX epoch: {e}"))?;

        i64::try_from(dur.as_nanos())
            .map_err(|e| anyhow!("timestamp overflow: {e}"))
    }
}
