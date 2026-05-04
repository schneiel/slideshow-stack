//! Media Business Logic
//!
//! Pure functions for media file operations.

use std::path::Path;

use anyhow::Context;

use super::types::{MediaMetadata, UploadResponse, DeleteMediaResponse};

pub type Result<T> = anyhow::Result<T>;

const MAX_SIZE: usize = 500 * 1024 * 1024;

/// # Errors
/// Returns errors from filesystem operations.
pub async fn list_media(media_dir: &Path) -> Result<Vec<MediaMetadata>> {
    super::storage::list_media_files(media_dir)
        .await
        .context("failed to list media files")
}

/// Upload media files.
///
/// # Errors
/// Returns errors from filesystem operations or validation failures.
pub async fn upload_media(
    media_dir: &Path,
    files: Vec<(String, Vec<u8>)>,
) -> Result<UploadResponse> {
    let mut uploaded = Vec::new();
    let mut errors = Vec::new();

    for (filename, data) in files {
        match validate_upload(&filename, &data) {
            Ok(()) => match super::storage::save_file(media_dir, &filename, data).await {
                Ok(_) => uploaded.push(filename),
                Err(e) => errors.push(format!("{filename}: {e}")),
            },
            Err(e) => errors.push(format!("{filename}: {e}")),
        }
    }

    Ok(UploadResponse {
        uploaded_files: uploaded,
        upload_errors: errors,
    })
}

/// Delete a media file.
///
/// # Errors
/// Returns errors from filesystem operations.
pub async fn delete_media(
    media_dir: &Path,
    filename: &str,
) -> Result<DeleteMediaResponse> {
    super::storage::delete_file(media_dir, filename)
        .await
        .context("failed to delete media file")?;

    Ok(DeleteMediaResponse {
        message: "Media file deleted successfully".into(),
        filename: filename.into(),
    })
}

/// Get the path to a media file.
///
/// # Errors
/// Returns errors if file not found or path validation fails.
pub async fn get_media_path(
    media_dir: &Path,
    filename: &str,
) -> Result<std::path::PathBuf> {
    let path = super::storage::get_media_path(media_dir, filename)
        .context("failed to get media path")?;

    if !path.exists() {
        anyhow::bail!("media file not found: {filename}");
    }

    Ok(path)
}

fn validate_upload(filename: &str, data: &[u8]) -> Result<()> {
    super::storage::validate_filename(filename)
        .context("invalid filename")?;

    if data.len() > MAX_SIZE {
        anyhow::bail!("File too large (max 500MB)");
    }

    if let Some(ext) = Path::new(filename).extension() {
        let ext = ext.to_str()
            .ok_or_else(|| anyhow::anyhow!("Filename extension is not valid UTF-8"))?;
        let valid = matches!(
            ext.to_lowercase().as_str(),
            "png" | "jpg" | "jpeg" | "gif" | "mp4"
        );
        if !valid {
            anyhow::bail!("Unsupported file type");
        }
    } else {
        anyhow::bail!("No file extension");
    }

    Ok(())
}
