//! API Module
//!
//! HTTP API layer with routing, handlers, and middleware.
//! Provides REST endpoints compatible with the control-panel frontend.
//!
//! # Endpoints
//!
//! ## Health
//! - `GET /api/health` - Health check
//!
//! ## Slideshows
//! - `GET /api/slideshows` - List all slideshows
//! - `POST /api/slideshows` - Create slideshow
//! - `GET /api/slideshows/{id}` - Get slideshow by ID
//! - `PUT /api/slideshows/{id}` - Update slideshow
//! - `DELETE /api/slideshows/{id}` - Delete slideshow
//!
//! ## Media
//! - `GET /api/media` - List all media files
//! - `POST /api/media/upload` - Upload media files
//! - `GET /api/media/{filename}` - Download media file
//! - `DELETE /api/media/{filename}` - Delete media file

pub mod handlers;
pub mod responses;

use anyhow::Context;
use axum::{
    extract::DefaultBodyLimit,
    routing::{get, post},
    Router,
};
use sqlx::SqlitePool;
use std::path::PathBuf;
use tower_http::cors::CorsLayer;

pub use handlers::*;

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub media_dir: PathBuf,
}

/// Create and configure the API router.
///
/// # Errors
/// Returns an error if CORS is enabled but `CORS_ORIGIN` is not set or invalid.
pub fn create_router(
    pool: SqlitePool,
    media_dir: PathBuf,
    cors_enabled: bool,
    cors_origin: Option<String>,
) -> anyhow::Result<Router> {
    let state = AppState { pool, media_dir };

    let router = Router::new()
        .route("/api/health", get(health_handler))
        .route(
            "/api/slideshows",
            get(list_slideshows).post(create_slideshow),
        )
        .route(
            "/api/slideshows/{id}",
            get(get_slideshow)
                .put(update_slideshow)
                .delete(delete_slideshow),
        )
        .route("/api/media", get(list_media))
        .route("/api/media/upload", post(upload_media))
        .route(
            "/api/media/{filename}",
            get(get_media_file).delete(delete_media),
        )
        .with_state(state)
        .layer(DefaultBodyLimit::max(500 * 1024 * 1024));

    if cors_enabled {
        let origin = cors_origin
            .ok_or_else(|| anyhow::anyhow!("CORS_ORIGIN must be set when CORS_ENABLED is true"))?;
        let cors_layer = CorsLayer::new()
            .allow_origin(
                origin
                    .parse::<axum::http::HeaderValue>()
                    .context("CORS_ORIGIN must be a valid header value")?,
            )
            .allow_methods([
                axum::http::Method::GET,
                axum::http::Method::POST,
                axum::http::Method::PUT,
                axum::http::Method::DELETE,
                axum::http::Method::OPTIONS,
            ])
            .allow_headers([axum::http::header::CONTENT_TYPE]);
        Ok(router.layer(cors_layer))
    } else {
        Ok(router)
    }
}
