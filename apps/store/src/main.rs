//! Store API Server
//!
//! REST API server for slideshow and media management.
//! Compatible with the control-panel frontend.
//!
//! # Features
//!
//! - Slideshow CRUD operations
//! - Media file upload/download
//! - `SQLite` database with auto-migrations
//! - CORS support for frontend integration
//!
//! # Environment Variables
//!
//! - `PORT` - Server port (default: 61532)
//! - `DATABASE_URL` - Database URL (default: "sqlite:slideshows.db")
//! - `MEDIA_DIR` - Media directory (default: "./media")
//! - `RUST_LOG` - Log level (default: "info")

mod api;
mod config;
mod db;
mod media;
mod slideshow;

use std::net::SocketAddr;
use anyhow::Context;
use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use api::create_router;
use config::load_config;
use db::create_pool;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = load_config()
        .context("failed to load configuration")?;

    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&config.log_level));

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(filter)
        .init();

    info!("Starting Store API server on port {}", config.port);
    info!("Log level: {}, CORS enabled: {}", config.log_level, config.cors_enabled);

    info!("Connecting to database: {}", config.database_url);
    let pool = create_pool(&config.database_url)
        .await
        .context("failed to create database pool")?;
    info!("Database connection established");

    tokio::fs::create_dir_all(&config.media_dir)
        .await
        .context("failed to create media directory")?;
    info!("Media directory: {}", config.media_dir.display());

    let app = create_router(
        pool,
        config.media_dir,
        config.cors_enabled,
        config.cors_origin,
    )
    .context("failed to create router")?;

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    let listener = TcpListener::bind(addr)
        .await
        .context("failed to bind to address")?;

    info!("Server listening on http://{}", addr);
    info!("Endpoints:");
    info!("  GET    /api/health");
    info!("  GET    /api/slideshows");
    info!("  POST   /api/slideshows");
    info!("  GET    /api/slideshows/:id");
    info!("  PUT    /api/slideshows/:id");
    info!("  DELETE /api/slideshows/:id");
    info!("  GET    /api/media");
    info!("  POST   /api/media/upload");
    info!("  GET    /api/media/:filename");
    info!("  DELETE /api/media/:filename");

    axum::serve(listener, app).await?;

    Ok(())
}
