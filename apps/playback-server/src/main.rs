mod api;
mod config;
mod dispatcher;
mod state;
mod zenoh;

use anyhow::{Context, Result};
use api::create_router;
use config::Config;
use dispatcher::CommandDispatcher;
use serde_json::json;
use state::StateManager;
use std::sync::Arc;
use tracing::{debug, error, info, warn};
use ::zenoh::Session;
use zenoh::{KEY_SLIDESHOW_STATE_WILDCARD, KEY_VIDEO_STATE_WILDCARD};

async fn handle_state_message(
    payload: &[u8],
    state_manager: &std::sync::Arc<StateManager>,
    lifecycle_tx: &tokio::sync::broadcast::Sender<serde_json::Value>,
) {
    if let Some((device_id, _display_name, _state)) = state_manager.handle_state_message(payload).await
        && let Some(device_state) = state_manager.get_device_state(&device_id).await
    {
        let sse_event = json!({
            "event_type": "slideshow_state",
            "data": device_state,
        });
        if let Err(e) = lifecycle_tx.send(sse_event) {
            debug!(?e, "lifecycle event dropped, no receivers");
        }
    }
}

async fn handle_video_state_message(
    payload: &[u8],
    state_manager: &std::sync::Arc<StateManager>,
    lifecycle_tx: &tokio::sync::broadcast::Sender<serde_json::Value>,
) {
    if let Some((device_id, _display_name, _state)) = state_manager.handle_video_state_message(payload).await
        && let Some(device_state) = state_manager.get_device_state(&device_id).await
    {
        let sse_event = json!({
            "event_type": "video_state",
            "data": device_state,
        });
        if let Err(e) = lifecycle_tx.send(sse_event) {
            debug!(?e, "lifecycle event dropped, no receivers");
        }
    }
}

fn spawn_state_subscription_loop(
    session: Arc<Session>,
    state_manager: std::sync::Arc<StateManager>,
    lifecycle_tx: tokio::sync::broadcast::Sender<serde_json::Value>,
) {
    tokio::spawn(async move {
        let key_expr = KEY_SLIDESHOW_STATE_WILDCARD;
        let mut retry_count = 0;

        loop {
            match session.declare_subscriber(key_expr).await {
                Ok(subscriber) => {
                    info!("Subscribed to slideshow state updates: {}", key_expr);
                    retry_count = 0;
                    let sub = subscriber;

                    while let Ok(sample) = sub.recv_async().await {
                        let payload = sample.payload().to_bytes().to_vec();
                        handle_state_message(&payload, &state_manager, &lifecycle_tx).await;
                    }

                    warn!("State subscription lost, reconnecting...");
                }
                Err(e) => {
                    error!("Failed to subscribe to state updates: {}, retrying in 2s", e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                    retry_count += 1;
                    if retry_count > 100 {
                        error!("Too many subscription retries, giving up");
                        break;
                    }
                }
            }
        }
    });
}

fn spawn_video_state_subscription_loop(
    session: Arc<Session>,
    state_manager: std::sync::Arc<StateManager>,
    lifecycle_tx: tokio::sync::broadcast::Sender<serde_json::Value>,
) {
    tokio::spawn(async move {
        let key_expr = KEY_VIDEO_STATE_WILDCARD;
        let mut retry_count = 0;

        loop {
            match session.declare_subscriber(key_expr).await {
                Ok(subscriber) => {
                    info!("Subscribed to video state updates: {}", key_expr);
                    retry_count = 0;
                    let sub = subscriber;

                    while let Ok(sample) = sub.recv_async().await {
                        let payload = sample.payload().to_bytes().to_vec();
                        handle_video_state_message(&payload, &state_manager, &lifecycle_tx).await;
                    }

                    warn!("Video state subscription lost, reconnecting...");
                }
                Err(e) => {
                    error!("Failed to subscribe to video state updates: {}, retrying in 2s", e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                    retry_count += 1;
                    if retry_count > 100 {
                        error!("Too many video state subscription retries, giving up");
                        break;
                    }
                }
            }
        }
    });
}

#[tokio::main]
async fn main() -> Result<()> {
    if let Err(e) = dotenvy::dotenv() {
        tracing::debug!("dotenv file not loaded: {}", e);
    }

    let config = Config::from_env().context("failed to load configuration")?;

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(&config.log_level)),
        )
        .init();

    info!("Starting playback server, log level: {}", config.log_level);

    let session = Arc::new(
        zenoh::connect_with_retry(&config.zenoh_endpoint, None)
            .await
            .context("failed to connect to Zenoh")?,
    );

    let state_manager = std::sync::Arc::new(StateManager::new());
    let dispatcher = std::sync::Arc::new(CommandDispatcher::new(session.clone()));

    info!("State manager initialized");

    let (lifecycle_tx, _) = tokio::sync::broadcast::channel::<serde_json::Value>(1000);

    spawn_state_subscription_loop(
        session.clone(),
        state_manager.clone(),
        lifecycle_tx.clone(),
    );

    spawn_video_state_subscription_loop(
        session.clone(),
        state_manager.clone(),
        lifecycle_tx.clone(),
    );

    let app = create_router(
        api::ApiState {
            state_manager,
            dispatcher,
            lifecycle_tx: std::sync::Arc::new(lifecycle_tx),
        },
        config.cors_enabled,
        config.cors_origin.clone(),
    );

    let listener = tokio::net::TcpListener::bind(&config.http_bind_address()).await?;
    info!("HTTP server listening on {}", config.http_bind_address());

    axum::serve(listener, app).await?;

    info!("Shutdown complete");
    Ok(())
}