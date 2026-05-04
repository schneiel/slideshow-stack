//! Slideshow Business Logic
//!
//! Pure functions for slideshow CRUD operations.
//! No internal state - all dependencies passed as parameters.

use anyhow::Context;
use sqlx::SqlitePool;
use uuid::Uuid;

use super::types::{CreateSlideshowRequest, Slideshow, SlideshowSummary, UpdateSlideshowRequest};
use crate::db::{self, SlideshowRow};

pub type Result<T> = anyhow::Result<T>;

/// Create a new slideshow.
///
/// # Errors
/// Returns errors for validation failures or database errors.
pub async fn create_slideshow(
    pool: &SqlitePool,
    request: CreateSlideshowRequest,
) -> Result<Slideshow> {
    validate_create_request(&request).context("invalid create request")?;

    let id = Uuid::new_v4().to_string();

    let row = db::insert_slideshow(
        pool,
        &id,
        &request.name,
        request.description.as_deref(),
        request.interval_seconds.cast_signed(),
        request.loop_enabled,
        request.shuffle,
    )
    .await
    .context("failed to insert slideshow")?;

    if !request.media_ids.is_empty() {
        db::link_media_to_slideshow(pool, &id, &request.media_ids)
            .await
            .context("failed to link media to slideshow")?;
    }

    let media_ids = if request.media_ids.is_empty() {
        Vec::new()
    } else {
        request.media_ids.clone()
    };

    Ok(row_to_domain(row, media_ids))
}

/// Get a slideshow by ID.
///
/// # Errors
/// Returns errors if slideshow not found or database errors occur.
pub async fn get_slideshow(pool: &SqlitePool, id: &str) -> Result<Slideshow> {
    let row = db::get_slideshow(pool, id)
        .await
        .context("failed to fetch slideshow")?
        .ok_or_else(|| anyhow::anyhow!("slideshow not found: {id}"))?;

    let media_ids = db::get_slideshow_media_ids(pool, id)
        .await
        .context("failed to get slideshow media ids")?;

    Ok(row_to_domain(row, media_ids))
}

/// List all slideshows.
///
/// # Errors
/// Returns errors from database operations.
pub async fn list_slideshows(pool: &SqlitePool) -> Result<Vec<SlideshowSummary>> {
    let rows = db::list_slideshows(pool)
        .await
        .context("failed to list slideshows")?;

    let mut summaries = Vec::new();
    for row in rows {
        let media_ids = db::get_slideshow_media_ids(pool, &row.id)
            .await
            .context("failed to get slideshow media ids")?;

        summaries.push(SlideshowSummary {
            id: row.id,
            name: row.name,
            description: row.description,
            media_ids,
            interval_seconds: row.interval_seconds.cast_unsigned(),
            loop_enabled: row.loop_enabled,
            shuffle: row.shuffle,
            created_at: row.created_at,
        });
    }

    Ok(summaries)
}

/// Update a slideshow.
///
/// # Errors
/// Returns errors if slideshow not found, validation fails, or database errors occur.
pub async fn update_slideshow(
    pool: &SqlitePool,
    id: &str,
    request: UpdateSlideshowRequest,
) -> Result<Slideshow> {
    let existing = db::get_slideshow(pool, id)
        .await
        .context("failed to fetch slideshow")?
        .ok_or_else(|| anyhow::anyhow!("slideshow not found: {id}"))?;

    let name = request.name.unwrap_or(existing.name);
    let description = request.description.or(existing.description);
    let interval_seconds = request
        .interval_seconds
        .unwrap_or_else(|| existing.interval_seconds.cast_unsigned());
    let loop_enabled = request.loop_enabled.unwrap_or(existing.loop_enabled);
    let shuffle = request.shuffle.unwrap_or(existing.shuffle);

    if name.trim().is_empty() {
        anyhow::bail!("name cannot be empty");
    }
    if !(1..=30).contains(&interval_seconds) {
        anyhow::bail!("interval must be between 1 and 30 seconds");
    }

    let row = db::update_slideshow(
        pool,
        id,
        &name,
        description.as_deref(),
        interval_seconds.cast_signed(),
        loop_enabled,
        shuffle,
    )
    .await
    .context("failed to update slideshow")?;

    let media_ids = if let Some(new_media_ids) = request.media_ids {
        db::update_slideshow_media(pool, id, &new_media_ids)
            .await
            .context("failed to update slideshow media")?;
        new_media_ids
    } else {
        db::get_slideshow_media_ids(pool, id)
            .await
            .context("failed to get slideshow media ids")?
    };

    Ok(row_to_domain(row, media_ids))
}

/// Delete a slideshow.
///
/// # Errors
/// Returns errors if slideshow not found or database errors occur.
pub async fn delete_slideshow(pool: &SqlitePool, id: &str) -> Result<()> {
    let affected = db::delete_slideshow(pool, id)
        .await
        .context("failed to delete slideshow")?;

    if affected == 0 {
        anyhow::bail!("slideshow not found: {id}");
    }

    Ok(())
}

fn validate_create_request(request: &CreateSlideshowRequest) -> Result<()> {
    let name = request.name.trim();
    if name.is_empty() {
        anyhow::bail!("Name cannot be empty");
    }
    if name.len() > 255 {
        anyhow::bail!("Name too long (max 255 characters)");
    }

    if !(1..=30).contains(&request.interval_seconds) {
        anyhow::bail!("Interval must be between 1 and 30 seconds");
    }

    if request.media_ids.len() > 1000 {
        anyhow::bail!("Too many media files (max 1000)");
    }

    Ok(())
}

fn row_to_domain(row: SlideshowRow, media_ids: Vec<String>) -> Slideshow {
    Slideshow {
        id: row.id,
        name: row.name,
        description: row.description,
        media_ids,
        interval_seconds: row.interval_seconds.cast_unsigned(),
        loop_enabled: row.loop_enabled,
        shuffle: row.shuffle,
        created_at: row.created_at,
        updated_at: row.updated_at,
    }
}
