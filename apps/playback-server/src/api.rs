use crate::dispatcher::{CommandDispatcher, ZenohCommand};
use crate::state::StateManager;
use axum::{
    Router,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json, Sse, sse::Event},
    routing::{get, post},
};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::broadcast;
use tower_http::cors::CorsLayer;
use tracing::{debug, error, info};

/// Shared state for all API handlers.
///
/// Contains references to the state manager, command dispatcher,
/// and lifecycle event broadcaster. Thread-safe via Arc.
pub struct ApiState {
    /// Manager for slideshow state across devices
    pub state_manager: Arc<StateManager>,
    /// Dispatches commands to devices via Zenoh
    pub dispatcher: Arc<CommandDispatcher>,
    /// Broadcasts lifecycle events to SSE subscribers
    pub lifecycle_tx: Arc<broadcast::Sender<serde_json::Value>>,
}

impl Clone for ApiState {
    fn clone(&self) -> Self {
        Self {
            state_manager: self.state_manager.clone(),
            dispatcher: self.dispatcher.clone(),
            lifecycle_tx: self.lifecycle_tx.clone(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub time: String,
}

#[derive(Debug, Deserialize)]
pub struct StartSlideshowRequest {
    #[serde(default)]
    pub target_device_ids: Vec<String>,
    pub name: Option<String>,
    pub media_files: Vec<String>,
    pub interval_seconds: f64,
    pub shuffle_enabled: bool,
    pub scaling_mode: Option<String>,
    pub loop_enabled: bool,
}

#[derive(Debug, Deserialize)]
pub struct StopSlideshowRequest {
    #[serde(default)]
    pub target_device_ids: Vec<String>,
    #[serde(default)]
    pub graceful: bool,
}

#[derive(Debug, Deserialize)]
pub struct TargetClientRequest {
    #[serde(default)]
    pub target_device_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct SetScalingRequest {
    #[serde(default)]
    pub target_device_ids: Vec<String>,
    pub mode: i32,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAutostartConfigRequest {
    #[serde(default)]
    pub target_device_ids: Vec<String>,
    pub autostart_config: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct StartVideoRequest {
    #[serde(default)]
    pub target_device_ids: Vec<String>,
    pub filename: String,
}

#[derive(Debug, Deserialize)]
pub struct VideoControlRequest {
    #[serde(default)]
    pub target_device_ids: Vec<String>,
}

/// Creates the Axum router with all API routes and SSE endpoint.
///
/// # Arguments
///
/// * `state` - Shared API state
/// * `cors_enabled` - Whether to enable CORS
/// * `cors_origin` - Allowed origin for CORS (if enabled)
///
/// # Returns
///
/// Configured Axum Router with all slideshow control endpoints
pub fn create_router(
    state: ApiState,
    cors_enabled: bool,
    cors_origin: Option<axum::http::HeaderValue>,
) -> Router {
    let router = Router::new()
        .route("/api/health", get(health))
        .route("/api/slideshow/start", post(start_slideshow))
        .route("/api/slideshow/stop", post(stop_slideshow))
        .route("/api/slideshow/pause", post(pause_slideshow))
        .route("/api/slideshow/resume", post(resume_slideshow))
        .route("/api/slideshow/next", post(next_image))
        .route("/api/slideshow/previous", post(previous_image))
        .route("/api/slideshow/set-scaling", post(set_scaling))
        .route("/api/slideshow/update-autostart", post(update_autostart))
        .route("/api/video/start", post(start_video))
        .route("/api/video/stop", post(stop_video))
        .route("/api/video/pause", post(pause_video))
        .route("/api/video/resume", post(resume_video))
        .route("/api/video/set-scaling", post(set_video_scaling))
        .route("/api/clients", get(list_clients))
        .route("/stream/events", get(event_stream))
        .with_state(state);

    if cors_enabled {
        if let Some(origin) = cors_origin {
            let cors = CorsLayer::new()
                .allow_origin(origin)
                .allow_methods([
                    axum::http::Method::GET,
                    axum::http::Method::POST,
                    axum::http::Method::OPTIONS,
                ])
                .allow_headers([axum::http::header::CONTENT_TYPE]);
            router.layer(cors)
        } else {
            router
        }
    } else {
        router
    }
}

async fn health() -> impl IntoResponse {
    Json(HealthResponse {
        status: "ok".to_string(),
        time: chrono::Utc::now().to_rfc3339(),
    })
}

async fn start_slideshow(
    State(state): State<ApiState>,
    Json(req): Json<StartSlideshowRequest>,
) -> impl IntoResponse {
    let config = serde_json::json!({
        "name": req.name,
        "media_files": req.media_files,
        "interval_seconds": req.interval_seconds,
        "shuffle_enabled": req.shuffle_enabled,
        "scaling_mode": req.scaling_mode,
        "loop_enabled": req.loop_enabled,
    });

    dispatch_command(&state, "start", req.target_device_ids, Some(config)).await
}

async fn stop_slideshow(
    State(state): State<ApiState>,
    Json(req): Json<StopSlideshowRequest>,
) -> impl IntoResponse {
    let config = serde_json::json!({
        "graceful": req.graceful,
    });

    dispatch_command(&state, "stop_slideshow", req.target_device_ids, Some(config)).await
}

async fn pause_slideshow(
    State(state): State<ApiState>,
    Json(req): Json<TargetClientRequest>,
) -> impl IntoResponse {
    dispatch_command(&state, "pause_slideshow", req.target_device_ids, None).await
}

async fn resume_slideshow(
    State(state): State<ApiState>,
    Json(req): Json<TargetClientRequest>,
) -> impl IntoResponse {
    dispatch_command(&state, "resume_slideshow", req.target_device_ids, None).await
}

async fn next_image(
    State(state): State<ApiState>,
    Json(req): Json<TargetClientRequest>,
) -> impl IntoResponse {
    dispatch_command(&state, "next", req.target_device_ids, None).await
}

async fn previous_image(
    State(state): State<ApiState>,
    Json(req): Json<TargetClientRequest>,
) -> impl IntoResponse {
    dispatch_command(&state, "previous", req.target_device_ids, None).await
}

async fn set_scaling(
    State(state): State<ApiState>,
    Json(req): Json<SetScalingRequest>,
) -> impl IntoResponse {
    let config = serde_json::json!({ "mode": req.mode });
    dispatch_command(&state, "set_slideshow_scaling_mode", req.target_device_ids, Some(config)).await
}

async fn update_autostart(
    State(state): State<ApiState>,
    Json(req): Json<UpdateAutostartConfigRequest>,
) -> impl IntoResponse {
    let config = match serde_json::to_value(req.autostart_config) {
        Ok(v) => Some(v),
        Err(e) => {
            error!("Failed to serialize autostart config: {}", e);
            return StatusCode::BAD_REQUEST;
        }
    };
    dispatch_command(&state, "set_config", req.target_device_ids, config).await
}

async fn start_video(
    State(state): State<ApiState>,
    Json(req): Json<StartVideoRequest>,
) -> impl IntoResponse {
    let config = serde_json::json!({
        "filename": req.filename,
    });
    dispatch_command(&state, "start_video", req.target_device_ids, Some(config)).await
}

async fn stop_video(
    State(state): State<ApiState>,
    Json(req): Json<VideoControlRequest>,
) -> impl IntoResponse {
    dispatch_command(&state, "stop_video", req.target_device_ids, None).await
}

async fn pause_video(
    State(state): State<ApiState>,
    Json(req): Json<VideoControlRequest>,
) -> impl IntoResponse {
    dispatch_command(&state, "pause_video", req.target_device_ids, None).await
}

async fn resume_video(
    State(state): State<ApiState>,
    Json(req): Json<VideoControlRequest>,
) -> impl IntoResponse {
    dispatch_command(&state, "resume_video", req.target_device_ids, None).await
}

async fn set_video_scaling(
    State(state): State<ApiState>,
    Json(req): Json<SetScalingRequest>,
) -> impl IntoResponse {
    let config = serde_json::json!({ "mode": req.mode });
    dispatch_command(&state, "set_video_scaling_mode", req.target_device_ids, Some(config)).await
}

async fn list_clients(State(state): State<ApiState>) -> impl IntoResponse {
    let devices = state.state_manager.list_all_devices().await;
    Json(devices)
}

async fn event_stream(State(state): State<ApiState>) -> impl IntoResponse {
    info!("SSE client connected to /stream/events");

    let devices = state.state_manager.list_all_devices().await;

    let mut initial_events: Vec<Event> = Vec::new();
    for device in &devices {
        let slideshow_event = serde_json::json!({
            "event_type": "slideshow_state",
            "data": device
        });
        if let Ok(json) = serde_json::to_string(&slideshow_event) {
            debug!("Sending slideshow_state for device_id={}", device.device_id);
            initial_events.push(Event::default().data(json));
        }

        if device.video_state.is_some() {
            let video_event = serde_json::json!({
                "event_type": "video_state",
                "data": device
            });
            if let Ok(json) = serde_json::to_string(&video_event) {
                debug!("Sending video_state for device_id={}", device.device_id);
                initial_events.push(Event::default().data(json));
            }
        }
    }

    let lifecycle_rx = state.lifecycle_tx.subscribe();

    let lifecycle_stream =
        tokio_stream::wrappers::BroadcastStream::new(lifecycle_rx).filter_map(|event| async move {
            event.ok().and_then(|e| {
                serde_json::to_string(&e)
                    .map_err(|e| tracing::warn!("Failed to serialize lifecycle event: {e}"))
                    .ok()
                    .map(|json| Event::default().data(json))
            })
        });

    let stream = futures::stream::iter(
        initial_events
            .into_iter()
            .map(Ok::<_, std::convert::Infallible>),
    )
    .chain(lifecycle_stream.map(Ok::<_, std::convert::Infallible>));

    info!("SSE stream established, keeping alive");
    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new().interval(std::time::Duration::from_secs(10)),
    )
}

async fn dispatch_command(
    state: &ApiState,
    command: &str,
    target_device_ids: Vec<String>,
    config: Option<serde_json::Value>,
) -> StatusCode {
    info!("Command received: {}", command);

    if !target_device_ids.is_empty() {
        state
            .dispatcher
            .dispatch_batch(command, target_device_ids, config)
            .await;
        return StatusCode::ACCEPTED;
    }

    let all_devices = state.state_manager.list_all_devices().await;
    let all_device_ids: Vec<String> = all_devices.into_iter().map(|d| d.device_id).collect();

    if all_device_ids.is_empty() {
        debug!("No devices registered, skipping broadcast command");
        return StatusCode::ACCEPTED;
    }

    let zenoh_command = match ZenohCommand::builder()
        .command(command)
        .broadcast()
        .config_opt(config)
        .try_build()
    {
        Ok(cmd) => cmd,
        Err(e) => {
            error!("Failed to build broadcast command: {}", e);
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    };

    state.dispatcher.dispatch(zenoh_command, all_device_ids).await;

    StatusCode::ACCEPTED
}