use crate::sync::ServerMediaFile;
use anyhow::{Context, Result, bail};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;
use tracing::info;

pub fn is_file_valid(filename: &str, server_file: &ServerMediaFile, media_dir: &Path) -> bool {
    let local_path = media_dir.join(filename);

    let Ok(file_info) = fs::metadata(&local_path) else {
        return false;
    };

    if file_info.len().cast_signed() != server_file.size {
        info!(
            "Size mismatch for {}: local={}, server={}",
            filename,
            file_info.len(),
            server_file.size
        );
        return false;
    }

    if !server_file.hash.is_empty() {
        match calculate_file_hash(&local_path) {
            Ok(local_hash) => {
                if local_hash != server_file.hash {
                    info!(
                        "Hash mismatch for {}: local={}, server={}",
                        filename, local_hash, server_file.hash
                    );
                    return false;
                }
            }
            Err(err) => {
                info!("Failed to calculate hash for {}: {}", filename, err);
                return false;
            }
        }
    }

    true
}

pub fn validate_downloaded_file(local_path: &Path, server_file: &ServerMediaFile) -> Result<()> {
    let file_info = fs::metadata(local_path).context("Failed to stat downloaded file")?;

    if file_info.len().cast_signed() != server_file.size {
        bail!(
            "Size mismatch: downloaded={}, expected={}",
            file_info.len(),
            server_file.size
        );
    }

    if !server_file.hash.is_empty() {
        let content = fs::read(local_path)?;
        let mut hasher = Sha256::new();
        hasher.update(&content);
        let local_hash = format!("{:x}", hasher.finalize());

        if local_hash != server_file.hash {
            bail!(
                "Hash mismatch: downloaded={}, expected={}",
                local_hash,
                server_file.hash
            );
        }
    }

    Ok(())
}

fn calculate_file_hash(file_path: &Path) -> Result<String> {
    let content = fs::read(file_path).context("Failed to read file for hashing")?;

    let mut hasher = Sha256::new();
    hasher.update(&content);
    let result = hasher.finalize();

    Ok(format!("{result:x}"))
}
