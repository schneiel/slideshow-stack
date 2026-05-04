//! Media Domain Types
//!
//! Domain models for media file management.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Media file metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaMetadata {
    /// Filename without path
    pub filename: String,
    /// Full path to the file
    pub path: String,
    /// File size in bytes
    pub size: u64,
    /// Last modification time
    pub mod_time: DateTime<Utc>,
    /// Type of media file
    pub media_type: MediaType,
    /// SHA256 hash of file contents (hex-encoded)
    #[serde(skip_serializing_if = "String::is_empty")]
    pub hash: String,
}

/// Type of media file.
///
/// Determined by file extension.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MediaType {
    /// Image file (png, jpg, jpeg, gif)
    Image,
    /// Video file (mp4)
    Video,
}

/// Response after uploading media files.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadResponse {
    /// Successfully uploaded filenames
    pub uploaded_files: Vec<String>,
    /// Error messages for failed uploads
    pub upload_errors: Vec<String>,
}

/// Response after deleting a media file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteMediaResponse {
    /// Success message
    pub message: String,
    /// Filename that was deleted
    pub filename: String,
}
