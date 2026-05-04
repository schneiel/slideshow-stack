//! Database Module
//!
//! Database connection, migrations, and query interface.
//! Uses `SQLite` with sqlx for compile-time checked queries.

pub mod models;
pub mod queries;

use anyhow::Context;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::{sqlite::SqliteConnectOptions, SqlitePool};
use std::str::FromStr;
use tracing::info;

pub use models::SlideshowRow;
pub use queries::*;

pub type Result<T> = anyhow::Result<T>;

/// Create a database connection pool.
///
/// # Errors
/// Returns errors for invalid database URLs or connection failures.
pub async fn create_pool(database_url: &str) -> Result<SqlitePool> {
    info!("Connecting to database: {}", database_url);

    let options = SqliteConnectOptions::from_str(database_url)
        .map_err(|e| anyhow::anyhow!("invalid database URL: {e}"))?
        .create_if_missing(true)
        .busy_timeout(std::time::Duration::from_secs(30))
        .pragma("journal_mode", "WAL")
        .pragma("foreign_keys", "true");

    let pool = SqlitePoolOptions::new()
        .max_connections(10)
        .connect_with(options)
        .await
        .context("failed to connect to database")?;

    info!("Running database migrations");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .context("failed to run database migrations")?;

    info!("Database connection established and migrations applied");

    Ok(pool)
}
