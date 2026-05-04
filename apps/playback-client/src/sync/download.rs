use crate::sync::ServerMediaFile;
use anyhow::{Result, bail};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;
use tokio::fs;
use tokio::sync::{Mutex as TokioMutex, Semaphore};
use tracing::{debug, warn};

use super::validation::validate_downloaded_file;

pub async fn download_files_in_parallel(
    source_server: &str,
    media_dir: &Path,
    filenames: Vec<String>,
    server_file_map: &HashMap<String, ServerMediaFile>,
    max_concurrent_downloads: usize,
    sync_cache: Arc<TokioMutex<HashMap<String, Instant>>>,
) -> Result<()> {
    let total_count = filenames.len();
    let semaphore = Arc::new(Semaphore::new(max_concurrent_downloads));
    let downloaded_files = Arc::new(TokioMutex::new(Vec::new()));
    let error_files = Arc::new(TokioMutex::new(Vec::new()));

    let mut tasks = Vec::new();

    for filename in filenames {
        let server_file = server_file_map[&filename].clone();
        let source_server = source_server.to_string();
        let media_dir = media_dir.to_path_buf();
        let downloaded = Arc::clone(&downloaded_files);
        let errors = Arc::clone(&error_files);
        let permit = Arc::clone(&semaphore);

        let task = tokio::spawn(async move {
            let _permit = match permit.acquire().await {
                Ok(p) => p,
                Err(e) => {
                    return Err(anyhow::anyhow!("Semaphore acquire failed: {filename}: {e}"));
                }
            };

            if let Err(err) =
                download_single_file(&source_server, &media_dir, &filename, &server_file).await
            {
                errors.lock().await.push(format!("{filename}: {err}"));
                return Ok(());
            }

            downloaded.lock().await.push(filename);
            Ok(())
        });

        tasks.push(task);
    }

    for task in tasks {
        let result = task.await?;
        result?;
    }

    let downloaded_count = downloaded_files.lock().await.len();
    {
        let mut cache = sync_cache.lock().await;
        let now = Instant::now();
        for file in downloaded_files.lock().await.iter() {
            cache.insert(file.clone(), now);
        }
    }

    {
        let errors = error_files.lock().await;
        let (errors_len, err_msg) = if errors.is_empty() {
            (0, None)
        } else {
            (errors.len(), Some(errors.join("; ")))
        };
        drop(errors);
        if let Some(err_msg) = err_msg {
            warn!(
                "Partial download success: {} of {} files succeeded, {} failed: {}",
                downloaded_count, total_count, errors_len, err_msg
            );
            bail!("Download errors: {err_msg}");
        }
    }

    Ok(())
}

async fn download_single_file(
    source_server: &str,
    media_dir: &Path,
    filename: &str,
    server_file: &ServerMediaFile,
) -> Result<()> {
    debug!("Downloading {} (size: {})", filename, server_file.size);

    let local_path = media_dir.join(filename);
    let remote_url = format!("{source_server}/api/media/{filename}");

    let response = reqwest::get(&remote_url).await?;

    if response.status().as_u16() != 200 {
        bail!("Server returned status {}", response.status());
    }

    let bytes = response.bytes().await?;

    if let Some(parent) = local_path.parent() {
        fs::create_dir_all(parent).await?;
    }

    fs::write(&local_path, bytes).await?;

    validate_downloaded_file(&local_path, server_file)?;

    Ok(())
}

pub async fn cleanup_obsolete_files(
    media_dir: &Path,
    server_file_map: &HashMap<String, ServerMediaFile>,
) -> Result<()> {
    debug!("Checking for obsolete files");

    let obsolete_files: Vec<PathBuf> = scan_local_files(media_dir)
        .await?
        .into_iter()
        .filter(|f| !server_file_map.contains_key(f))
        .map(|f| media_dir.join(&f))
        .collect();

    if obsolete_files.is_empty() {
        debug!("No obsolete files");
        return Ok(());
    }

    debug!("Removing {} obsolete files", obsolete_files.len());

    for file_path in obsolete_files {
        if let Err(err) = fs::remove_file(&file_path).await {
            warn!("Failed to remove file {:?}: {}", file_path, err);
        }
    }

    Ok(())
}

async fn scan_local_files(media_dir: &Path) -> Result<Vec<String>> {
    let mut entries = fs::read_dir(media_dir).await?;
    let mut files = Vec::new();
    while let Some(entry) = entries.next_entry().await? {
        if !entry.path().is_dir() && let Some(name) = entry.file_name().to_str() {
            files.push(name.to_string());
        }
    }
    Ok(files)
}
