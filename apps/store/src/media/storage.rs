//! Media File Storage
//!
//! Filesystem operations for media files.

use anyhow::Context;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::info;

use super::types::MediaMetadata;

pub type Result<T> = anyhow::Result<T>;

/// Lists all media files in the specified directory.
///
/// # Arguments
///
/// * `media_dir` - Path to media directory
///
/// # Returns
///
/// Vector of media metadata for all valid media files
///
/// # Errors
///
/// Returns `ServiceError::Io` if directory cannot be read
pub async fn list_media_files(media_dir: &Path) -> Result<Vec<MediaMetadata>> {
    let mut entries = fs::read_dir(media_dir)
        .await
        .context("failed to read media directory")?;

    let mut files = Vec::new();

    while let Some(entry) = entries.next_entry().await
        .context("failed to read directory entry")?
    {
        let path = entry.path();

        if path.is_dir() {
            continue;
        }

        let Some(metadata) = process_entry(&path).await else {
            continue;
        };

        files.push(metadata);
    }

    files.sort_by(|a, b| a.filename.cmp(&b.filename));

    Ok(files)
}

async fn process_entry(path: &Path) -> Option<MediaMetadata> {
    let ext = path.extension()?.to_str()?;
    if !is_media_extension(ext) {
        return None;
    }

    let entry_metadata = tokio::fs::metadata(path).await.ok()?;
    let mod_time = entry_metadata.modified().ok()?.into();

    let filename = path.file_name()?.to_str()?;
    let media_type = get_media_type(ext);

    let hash = calculate_hash(path).await.ok()?;

    Some(MediaMetadata {
        filename: filename.to_string(),
        path: path.to_string_lossy().to_string(),
        size: entry_metadata.len(),
        mod_time,
        media_type,
        hash,
    })
}

/// Gets the full path for a media file.
///
/// # Arguments
///
/// * `media_dir` - Base media directory
/// * `filename` - Name of the file
///
/// # Returns
///
/// Full path to the file
///
/// # Errors
///
/// Returns `ServiceError::InvalidInput` if filename is invalid
pub fn get_media_path(media_dir: &Path, filename: &str) -> Result<PathBuf> {
    validate_filename(filename)?;

    let path = media_dir.join(filename);

    let canonical_path = path.canonicalize()
        .context("failed to canonicalize path")?;
    let canonical_dir = media_dir.canonicalize()
        .context("failed to canonicalize media directory")?;

    if !canonical_path.starts_with(&canonical_dir) {
        anyhow::bail!("Path traversal detected");
    }

    Ok(path)
}

/// Saves an uploaded file to the media directory.
///
/// # Arguments
///
/// * `media_dir` - Base media directory
/// * `filename` - Name for the file
/// * `data` - File contents
///
/// # Returns
///
/// Full path to saved file
///
/// # Errors
///
/// - `ServiceError::InvalidInput` if filename is invalid
/// - `ServiceError::Io` if write fails
pub async fn save_file(
    media_dir: &Path,
    filename: &str,
    data: Vec<u8>,
) -> Result<PathBuf> {
    validate_filename(filename)?;

    let path = media_dir.join(filename);

    fs::create_dir_all(media_dir).await
        .context("failed to create media directory")?;

    fs::write(&path, data).await
        .context("failed to write file")?;

    info!("Saved media file: {}", path.display());

    Ok(path)
}

/// Deletes a media file from the filesystem.
///
/// # Arguments
///
/// * `media_dir` - Base media directory
/// * `filename` - Name of file to delete
///
/// # Returns
///
/// Unit on success
///
/// # Errors
///
/// - `ServiceError::NotFound` if file doesn't exist
/// - `ServiceError::Io` if deletion fails
pub async fn delete_file(media_dir: &Path, filename: &str) -> Result<()> {
    let path = get_media_path(media_dir, filename)?;

    if !path.exists() {
        anyhow::bail!("media file not found: {filename}");
    }

    fs::remove_file(&path).await
        .context("failed to delete file")?;

    info!("Deleted media file: {}", path.display());

    Ok(())
}

/// Checks if a file extension is a valid media type.
///
/// # Arguments
///
/// * `ext` - File extension (without dot)
///
/// # Returns
///
/// true if extension is supported
fn is_media_extension(ext: &str) -> bool {
    matches!(
        ext.to_lowercase().as_str(),
        "png" | "jpg" | "jpeg" | "gif" | "mp4"
    )
}

/// Determines media type from file extension.
///
/// # Arguments
///
/// * `ext` - File extension (without dot)
///
/// # Returns
///
/// Corresponding `MediaType`
fn get_media_type(ext: &str) -> super::types::MediaType {
    match ext.to_lowercase().as_str() {
        "mp4" => super::types::MediaType::Video,
        _ => super::types::MediaType::Image,
    }
}

/// Validates a filename for security.
///
/// # Arguments
///
/// * `filename` - Filename to validate
///
/// # Returns
///
/// Ok if valid
///
/// # Errors
///
/// Returns `ServiceError::InvalidInput` if filename contains dangerous characters
pub fn validate_filename(filename: &str) -> Result<()> {
    if filename.is_empty() {
        anyhow::bail!("Filename is empty");
    }

    if filename.contains("..") || filename.contains('/') || filename.contains('\\') {
        anyhow::bail!("Invalid filename");
    }

    if filename.len() > 255 {
        anyhow::bail!("Filename too long");
    }

    Ok(())
}

/// Calculates SHA256 hash for a file.
///
/// # Arguments
///
/// * `path` - Path to the file
///
/// # Returns
///
/// Hex-encoded SHA256 hash
///
/// # Errors
///
/// Returns `ServiceError::Io` if file cannot be read
async fn calculate_hash(path: &Path) -> Result<String> {
    let contents = fs::read(path).await
        .context("failed to read file for hashing")?;

    let mut hasher = Sha256::new();
    hasher.update(&contents);
    let result = hasher.finalize();

    Ok(format!("{result:x}"))
}
