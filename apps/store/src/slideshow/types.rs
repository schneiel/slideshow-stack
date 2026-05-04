//! Slideshow Domain Types
//!
//! Domain models for slideshow business logic.
//! These types are separate from database models and represent
//! the application's understanding of a slideshow.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Slideshow domain model.
///
/// Represents a complete slideshow with all its properties and associated media.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Slideshow {
    /// Unique identifier (UUID v4)
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Optional description
    pub description: Option<String>,
    /// Ordered list of media filenames
    pub media_ids: Vec<String>,
    /// Interval between slides in seconds (1-30)
    pub interval_seconds: u32,
    /// Whether to loop when reaching the end
    pub loop_enabled: bool,
    /// Whether to shuffle media order on playback
    pub shuffle: bool,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

/// Slideshow summary for list views.
///
/// Lighter version of Slideshow with essential media information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlideshowSummary {
    /// Unique identifier (UUID v4)
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Optional description
    pub description: Option<String>,
    /// Ordered list of media filenames
    pub media_ids: Vec<String>,
    /// Interval between slides in seconds (1-30)
    pub interval_seconds: u32,
    /// Whether to loop when reaching the end
    pub loop_enabled: bool,
    /// Whether to shuffle media order on playback
    pub shuffle: bool,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
}

/// Request to create a new slideshow.
#[derive(Debug, Deserialize)]
pub struct CreateSlideshowRequest {
    /// Human-readable name
    pub name: String,
    /// Optional description
    pub description: Option<String>,
    /// Ordered list of media filenames
    pub media_ids: Vec<String>,
    /// Interval between slides in seconds (1-30)
    #[serde(default = "default_interval")]
    pub interval_seconds: u32,
    /// Whether to loop when reaching the end
    #[serde(default)]
    pub loop_enabled: bool,
    /// Whether to shuffle media order on playback
    #[serde(default)]
    pub shuffle: bool,
}

/// Request to update an existing slideshow.
///
/// All fields are optional; only provided fields will be updated.
#[derive(Debug, Deserialize)]
pub struct UpdateSlideshowRequest {
    /// New human-readable name
    pub name: Option<String>,
    /// New optional description
    pub description: Option<String>,
    /// New ordered list of media filenames
    pub media_ids: Option<Vec<String>>,
    /// New interval between slides in seconds (1-30)
    pub interval_seconds: Option<u32>,
    /// New loop setting
    pub loop_enabled: Option<bool>,
    /// New shuffle setting
    pub shuffle: Option<bool>,
}

const fn default_interval() -> u32 {
    5
}
