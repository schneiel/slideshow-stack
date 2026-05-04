//! Database Queries
//!
//! SQL queries using standard sqlx methods.
//! All queries are prepared statements for performance.

use crate::db::models::SlideshowRow;
use anyhow::Context;
use sqlx::{Row, SqlitePool};

pub type Result<T> = anyhow::Result<T>;

/// # Errors
/// Returns an error if the database connection fails, the query fails,
/// or no slideshow with the given ID exists.
pub async fn get_slideshow(pool: &SqlitePool, id: &str) -> Result<Option<SlideshowRow>> {
    let row = sqlx::query(
        r"
        SELECT id, name, description, interval_seconds, loop_enabled, shuffle,
               created_at, updated_at
        FROM slideshows
        WHERE id = ?
        ",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .context("failed to fetch slideshow")?;

    row.map_or(Ok(None), |r| {
        Ok(Some(SlideshowRow {
            id: r.get("id"),
            name: r.get("name"),
            description: r.get("description"),
            interval_seconds: r.get("interval_seconds"),
            loop_enabled: r.get("loop_enabled"),
            shuffle: r.get("shuffle"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        }))
    })
}

/// # Errors
/// Returns an error if the database connection fails or the query fails.
pub async fn list_slideshows(pool: &SqlitePool) -> Result<Vec<SlideshowRow>> {
    let rows = sqlx::query(
        r"
        SELECT id, name, description, interval_seconds, loop_enabled, shuffle,
               created_at, updated_at
        FROM slideshows
        ORDER BY created_at DESC
        ",
    )
    .fetch_all(pool)
    .await
    .context("failed to list slideshows")?;

    let result = rows
        .into_iter()
        .map(|r| SlideshowRow {
            id: r.get("id"),
            name: r.get("name"),
            description: r.get("description"),
            interval_seconds: r.get("interval_seconds"),
            loop_enabled: r.get("loop_enabled"),
            shuffle: r.get("shuffle"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        })
        .collect();

    Ok(result)
}

/// # Errors
/// Returns an error if the database connection fails or the INSERT query fails.
pub async fn insert_slideshow(
    pool: &SqlitePool,
    id: &str,
    name: &str,
    description: Option<&str>,
    interval_seconds: i32,
    loop_enabled: bool,
    shuffle: bool,
) -> Result<SlideshowRow> {
    let row = sqlx::query(
        r"
        INSERT INTO slideshows (id, name, description, interval_seconds, loop_enabled, shuffle)
        VALUES (?, ?, ?, ?, ?, ?)
        RETURNING id, name, description, interval_seconds, loop_enabled, shuffle,
                  created_at, updated_at
        ",
    )
    .bind(id)
    .bind(name)
    .bind(description)
    .bind(interval_seconds)
    .bind(loop_enabled)
    .bind(shuffle)
    .fetch_one(pool)
    .await
    .context("failed to insert slideshow")?;

    Ok(SlideshowRow {
        id: row.get("id"),
        name: row.get("name"),
        description: row.get("description"),
        interval_seconds: row.get("interval_seconds"),
        loop_enabled: row.get("loop_enabled"),
        shuffle: row.get("shuffle"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}

/// # Errors
/// Returns an error if the database connection fails, the UPDATE query fails,
/// or no slideshow with the given ID exists.
pub async fn update_slideshow(
    pool: &SqlitePool,
    id: &str,
    name: &str,
    description: Option<&str>,
    interval_seconds: i32,
    loop_enabled: bool,
    shuffle: bool,
) -> Result<SlideshowRow> {
    let row = sqlx::query(
        r"
        UPDATE slideshows
        SET name = ?, description = ?, interval_seconds = ?,
            loop_enabled = ?, shuffle = ?
        WHERE id = ?
        RETURNING id, name, description, interval_seconds, loop_enabled, shuffle,
                  created_at, updated_at
        ",
    )
    .bind(name)
    .bind(description)
    .bind(interval_seconds)
    .bind(loop_enabled)
    .bind(shuffle)
    .bind(id)
    .fetch_one(pool)
    .await
    .context("failed to update slideshow")?;

    Ok(SlideshowRow {
        id: row.get("id"),
        name: row.get("name"),
        description: row.get("description"),
        interval_seconds: row.get("interval_seconds"),
        loop_enabled: row.get("loop_enabled"),
        shuffle: row.get("shuffle"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}

/// # Errors
/// Returns an error if the database connection fails or the DELETE query fails.
pub async fn delete_slideshow(pool: &SqlitePool, id: &str) -> Result<u64> {
    let result = sqlx::query(
        r"
        DELETE FROM slideshows
        WHERE id = ?
        ",
    )
    .bind(id)
    .execute(pool)
    .await
    .context("failed to delete slideshow")?;

    Ok(result.rows_affected())
}

/// # Errors
/// Returns an error if the database connection fails or the query fails.
pub async fn get_slideshow_media_ids(
    pool: &SqlitePool,
    slideshow_id: &str,
) -> Result<Vec<String>> {
    let rows = sqlx::query(
        r"
        SELECT media_id
        FROM slideshow_media
        WHERE slideshow_id = ?
        ORDER BY position ASC
        ",
    )
    .bind(slideshow_id)
    .fetch_all(pool)
    .await
    .context("failed to get slideshow media ids")?;

    Ok(rows.into_iter().map(|r| r.get("media_id")).collect())
}

/// Link media items to a slideshow.
///
/// # Errors
/// Returns an error if the database connection fails or the INSERT query fails.
pub async fn link_media_to_slideshow(
    pool: &SqlitePool,
    slideshow_id: &str,
    media_ids: &[String],
) -> Result<u64> {
    if media_ids.is_empty() {
        return Ok(0);
    }

    if media_ids.len() > i32::MAX as usize {
        anyhow::bail!("media_ids length {} exceeds i32::MAX", media_ids.len());
    }

    let mut affected = 0u64;

    for (position, media_id) in media_ids.iter().enumerate() {
        let link_id = uuid::Uuid::new_v4().to_string();

        let result = sqlx::query(
            r"
            INSERT INTO slideshow_media (id, slideshow_id, media_id, position)
            VALUES (?, ?, ?, ?)
            ",
        )
        .bind(&link_id)
        .bind(slideshow_id)
        .bind(media_id)
        .bind(i32::try_from(position)?)
        .execute(pool)
        .await
        .context("failed to link media to slideshow")?;

        affected += result.rows_affected();
    }

    Ok(affected)
}

/// # Errors
/// Returns an error if the database connection fails or the DELETE query fails.
pub async fn clear_slideshow_media(pool: &SqlitePool, slideshow_id: &str) -> Result<u64> {
    let result = sqlx::query(
        r"
        DELETE FROM slideshow_media
        WHERE slideshow_id = ?
        ",
    )
    .bind(slideshow_id)
    .execute(pool)
    .await
    .context("failed to clear slideshow media")?;

    Ok(result.rows_affected())
}

/// # Errors
/// Returns an error if the database connection fails or any of the queries fail.
pub async fn update_slideshow_media(
    pool: &SqlitePool,
    slideshow_id: &str,
    media_ids: &[String],
) -> Result<u64> {
    clear_slideshow_media(pool, slideshow_id).await?;
    link_media_to_slideshow(pool, slideshow_id, media_ids).await
}
