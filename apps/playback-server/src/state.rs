use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SlideshowState {
    pub status: String,
    pub name: Option<String>,
    pub image: Option<String>,
    pub interval: Option<i64>,
    pub scaling_mode: Option<String>,
    pub total_images: Option<i64>,
    pub current_index: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<serde_json::Map<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VideoState {
    pub status: String,
    pub filename: Option<String>,
    pub scaling_mode: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientStateUpdate {
    pub device_id: String,
    pub display_name: String,
    pub state: SlideshowState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientVideoStateUpdate {
    pub device_id: String,
    pub display_name: String,
    pub state: VideoState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub device_id: String,
    pub display_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceState {
    pub device_id: String,
    pub display_name: String,
    pub connected: bool,
    pub state: SlideshowState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub video_state: Option<VideoState>,
}

pub struct StateManager {
    slideshow_states: Arc<Mutex<HashMap<String, SlideshowState>>>,
    video_states: Arc<Mutex<HashMap<String, VideoState>>>,
    device_info: Arc<Mutex<HashMap<String, DeviceInfo>>>,
}

impl StateManager {
    pub fn new() -> Self {
        Self {
            slideshow_states: Arc::new(Mutex::new(HashMap::new())),
            video_states: Arc::new(Mutex::new(HashMap::new())),
            device_info: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn update_slideshow_state(&self, device_id: &str, state: SlideshowState) {
        let mut states = self.slideshow_states.lock().await;
        states.insert(device_id.to_string(), state);
    }

    pub async fn update_video_state(&self, device_id: &str, state: VideoState) {
        let mut states = self.video_states.lock().await;
        states.insert(device_id.to_string(), state);
    }

    pub async fn update_device_info(&self, device_id: &str, display_name: &str) {
        let mut info = self.device_info.lock().await;
        info.insert(
            device_id.to_string(),
            DeviceInfo {
                device_id: device_id.to_string(),
                display_name: display_name.to_string(),
            },
        );
    }

    pub async fn handle_state_message(
        &self,
        payload: &[u8],
    ) -> Option<(String, String, SlideshowState)> {
        let update: ClientStateUpdate = match serde_json::from_slice(payload) {
            Ok(s) => s,
            Err(e) => {
                tracing::warn!("Failed to parse slideshow state: {}", e);
                return None;
            }
        };

        self.update_device_info(&update.device_id, &update.display_name).await;
        self.update_slideshow_state(&update.device_id, update.state.clone())
            .await;
        Some((update.device_id, update.display_name, update.state))
    }

    pub async fn handle_video_state_message(
        &self,
        payload: &[u8],
    ) -> Option<(String, String, VideoState)> {
        let update: ClientVideoStateUpdate = match serde_json::from_slice(payload) {
            Ok(s) => s,
            Err(e) => {
                tracing::warn!("Failed to parse video state: {}", e);
                return None;
            }
        };

        self.update_device_info(&update.device_id, &update.display_name)
            .await;
        self.update_video_state(&update.device_id, update.state.clone())
            .await;
        Some((update.device_id, update.display_name, update.state))
    }

    pub async fn get_device_state(&self, device_id: &str) -> Option<DeviceState> {
        let slideshow_state = self
            .slideshow_states
            .lock()
            .await
            .get(device_id)
            .cloned();

        let video_state = self
            .video_states
            .lock()
            .await
            .get(device_id)
            .cloned();

        let device_info = self.device_info.lock().await.get(device_id).cloned();

        if slideshow_state.is_none() && video_state.is_none() {
            return None;
        }

        let (device_id, display_name) = device_info
            .map_or_else(
                || (device_id.to_string(), format!("Device {}", &device_id[..8.min(device_id.len())])),
                |info| (info.device_id, info.display_name),
            );

        Some(DeviceState {
            device_id,
            display_name,
            connected: true,
            state: slideshow_state.unwrap_or_default(),
            video_state,
        })
    }

    pub async fn list_all_devices(&self) -> Vec<DeviceState> {
        let device_ids: std::collections::HashSet<String> = self
            .slideshow_states
            .lock()
            .await
            .keys()
            .cloned()
            .chain(self.video_states.lock().await.keys().cloned())
            .collect();

        let mut devices = Vec::new();
        for device_id in device_ids {
            if let Some(state) = self.get_device_state(&device_id).await {
                devices.push(state);
            }
        }
        devices
    }
}

impl Default for StateManager {
    fn default() -> Self {
        Self::new()
    }
}