//! Database Models
//!
//! Exact mirror of the `SQLite` database schema.
//! All fields match the SQL schema structure (nullable/non-nullable).

use chrono::{DateTime, Utc};

/// Represents a slideshow in the database.
///
/// Corresponds to the `slideshows` table in the schema.
#[derive(Debug, Clone)]
pub struct SlideshowRow {
    /// UUID v4 primary key
    pub id: String,
    /// Slideshow name (NOT NULL)
    pub name: String,
    /// Optional description
    pub description: Option<String>,
    /// Interval in seconds (1-30)
    pub interval_seconds: i32,
    /// Whether the slideshow loops
    pub loop_enabled: bool,
    /// Whether media order is shuffled
    pub shuffle: bool,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}
