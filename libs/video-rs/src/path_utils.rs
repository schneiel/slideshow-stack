use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PathError {
    #[error("Filename cannot be empty")]
    EmptyFilename,
    #[error("Path traversal detected: filename contains '..'")]
    PathTraversal,
    #[error("Absolute paths are not allowed: '{0}'")]
    AbsolutePath(String),
    #[error("Nested paths are not allowed: '{0}'")]
    NestedPath(String),
    #[error("Failed to canonicalize path: {0}")]
    CanonicalizeFailed(#[from] std::io::Error),
}

/// # Errors
/// Returns `PathError` if the filename is empty, contains `..`, is absolute, or is nested.
pub fn validate_filename(filename: &str) -> Result<(), PathError> {
    if filename.is_empty() {
        return Err(PathError::EmptyFilename);
    }

    if filename.contains("..") {
        return Err(PathError::PathTraversal);
    }

    let path = Path::new(filename);

    if path.has_root() {
        return Err(PathError::AbsolutePath(filename.to_string()));
    }

    if path.components().count() > 1 {
        return Err(PathError::NestedPath(filename.to_string()));
    }

    Ok(())
}

/// # Errors
/// Returns `PathError` if any filename fails validation.
pub fn validate_filenames(filenames: &[String]) -> Result<(), PathError> {
    for filename in filenames {
        validate_filename(filename)?;
    }
    Ok(())
}

/// # Errors
/// Returns `PathError` if the filename is invalid or escapes the media directory.
pub fn sanitize_and_join(media_dir: &Path, filename: &str) -> Result<PathBuf, PathError> {
    if filename.is_empty() {
        return Err(PathError::EmptyFilename);
    }

    if filename.contains("..") {
        return Err(PathError::PathTraversal);
    }

    let path = Path::new(filename);

    if path.has_root() {
        return Err(PathError::AbsolutePath(filename.to_string()));
    }

    if path.components().count() > 1 {
        return Err(PathError::NestedPath(filename.to_string()));
    }

    let full_path = media_dir.join(filename);

    let canonical_media = media_dir.canonicalize()?;

    if full_path.exists() {
        let canonical_full = full_path.canonicalize()?;
        if !canonical_full.starts_with(&canonical_media) {
            return Err(PathError::PathTraversal);
        }
    } else {
        let full_str = full_path.to_string_lossy();
        if full_str.contains("..") {
            return Err(PathError::PathTraversal);
        }
    }

    Ok(full_path)
}

/// # Errors
/// Returns `PathError` if any filename fails validation or canonicalization.
pub fn sanitize_and_join_all(media_dir: &Path, filenames: &[String]) -> Result<Vec<PathBuf>, PathError> {
    filenames
        .iter()
        .map(|filename| sanitize_and_join(media_dir, filename))
        .collect()
}

#[must_use]
pub fn is_video_file(path: &str) -> bool {
    path.to_lowercase()
        .ends_with(".mp4")
        || path.to_lowercase().ends_with(".m4v")
        || path.to_lowercase().ends_with(".mov")
        || path.to_lowercase().ends_with(".avi")
        || path.to_lowercase().ends_with(".mkv")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_filename() {
        let media_dir = PathBuf::from("./media");
        let result = sanitize_and_join(&media_dir, "image.jpg");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), PathBuf::from("./media/image.jpg"));
    }

    #[test]
    fn test_empty_filename() {
        let media_dir = PathBuf::from("./media");
        let result = sanitize_and_join(&media_dir, "");
        assert!(result.is_err());
    }

    #[test]
    fn test_path_traversal_with_double_dot() {
        let media_dir = PathBuf::from("./media");
        let result = sanitize_and_join(&media_dir, "../etc/passwd");
        assert!(result.is_err());
    }

    #[test]
    fn test_is_video_file() {
        assert!(is_video_file("video.mp4"));
        assert!(is_video_file("video.MP4"));
        assert!(is_video_file("video.m4v"));
        assert!(is_video_file("video.mov"));
        assert!(is_video_file("video.avi"));
        assert!(is_video_file("video.mkv"));
        assert!(!is_video_file("video.mp3"));
        assert!(!is_video_file("image.jpg"));
    }
}