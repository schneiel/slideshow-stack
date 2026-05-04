use video_rs::path_utils::validate_filename;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex as TokioMutex;
use tracing::{debug, info, warn};

use self::client::MediaClient;
use self::download::{cleanup_obsolete_files, download_files_in_parallel};
use self::validation::is_file_valid;

/// Response from the media server listing available files.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ServerMediaList {
    success: bool,
    data: Vec<ServerMediaFile>,
}

/// Metadata for a media file on the server.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ServerMediaFile {
    filename: String,
    size: i64,
    hash: String,
}

/// Service for synchronizing media files from a server.
///
/// Uses a cache to avoid redundant downloads and validates file integrity
/// via hash comparison.
pub struct SyncService {
    source_server: String,
    media_dir: PathBuf,
    client: MediaClient,
    sync_cache: Arc<TokioMutex<HashMap<String, Instant>>>,
    max_concurrent_downloads: usize,
}

impl SyncService {
    /// Creates a new `SyncService` for downloading media files from a server.
    ///
    /// # Arguments
    ///
    /// * `source_server` - Base URL of the media server
    /// * `media_dir` - Local directory to store media files
    /// * `timeout_seconds` - HTTP request timeout
    ///
    /// # Errors
    ///
    /// Returns error if HTTP client cannot be created
    pub fn new(source_server: &str, media_dir: &str, timeout_seconds: u64) -> Result<Self> {
        let source_server = source_server.trim_end_matches('/').to_string();
        let media_dir = PathBuf::from(media_dir);

        let max_concurrent_downloads = env::var("MAX_CONCURRENT_DOWNLOADS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(2);

        let client = MediaClient::new(&source_server, timeout_seconds)
            .context("failed to create media sync HTTP client")?;

        Ok(Self {
            source_server,
            media_dir,
            client,
            sync_cache: Arc::new(TokioMutex::new(HashMap::new())),
            max_concurrent_downloads,
        })
    }

    /// Downloads media files that are missing or invalid locally.
    ///
    /// Compares requested files against the server and local state,
    /// then downloads any missing or outdated files.
    ///
    /// # Arguments
    ///
    /// * `requested_files` - List of filenames to sync
    ///
    /// # Errors
    ///
    /// Returns error if server is unreachable and files are missing locally
    pub async fn download_missing_media(&self, requested_files: Vec<String>) -> Result<()> {
        if requested_files.is_empty() {
            debug!("No media files to sync");
            return Ok(());
        }

        for filename in &requested_files {
            validate_filename(filename)
                .with_context(|| format!("Invalid filename in media sync request: {filename}"))?;
        }

        debug!("Starting media sync for {} files", requested_files.len());

        let server_files = match self.client.fetch_server_media_list().await {
            Ok(files) => files,
            Err(err) => {
                return self.handle_offline_mode(&requested_files, &err);
            }
        };

        let server_file_map: HashMap<String, ServerMediaFile> = server_files
            .into_iter()
            .map(|f| (f.filename.clone(), f))
            .collect();

        let files_to_download = self
            .analyze_sync_state(&requested_files, &server_file_map)
            .await;

        if let Err(err) = cleanup_obsolete_files(&self.media_dir, &server_file_map).await {
            warn!("Cleanup of obsolete files failed: {}", err);
        }

        if files_to_download.is_empty() {
            debug!("All requested files are already up to date");
            return Ok(());
        }

        debug!("Downloading {} files", files_to_download.len());
        debug!("File list: {:?}", files_to_download);

        self.download_files_in_parallel(files_to_download, &server_file_map)
            .await?;

        info!("Successfully synced files");
        Ok(())
    }

    fn get_local_media_list(&self) -> Result<Vec<String>> {
        Ok(fs::read_dir(&self.media_dir)
            .context("Failed to read media directory")?
            .filter_map(std::result::Result::ok)
            .filter(|entry| !entry.path().is_dir())
            .filter_map(|entry| entry.file_name().to_str().map(String::from))
            .collect())
    }

    fn handle_offline_mode(
        &self,
        requested_files: &[String],
        server_err: &anyhow::Error,
    ) -> Result<()> {
        warn!(
            "Server offline: {}, checking for local media files",
            server_err
        );

        let local_files = self
            .get_local_media_list()
            .context("Server offline and no local media available")?;

        let missing_files = find_missing_files(requested_files, &local_files);

        if missing_files.is_empty() {
            info!("All requested files available locally - continuing in offline mode");
        } else {
            warn!(
                "Server offline - {} files not available locally",
                missing_files.len()
            );
            debug!("Missing files: {:?}", missing_files);
            info!("Continuing with available local files only");
        }

        Ok(())
    }

    async fn analyze_sync_state(
        &self,
        requested_files: &[String],
        server_file_map: &HashMap<String, ServerMediaFile>,
    ) -> Vec<String> {
        let mut cache = self.sync_cache.lock().await;
        let now = Instant::now();
        let recent_threshold = Duration::from_mins(5);

        let files_needing_download: Vec<String> = requested_files
            .iter()
            .filter(|filename| {
                should_download_file(
                    filename,
                    server_file_map,
                    &cache,
                    now,
                    recent_threshold,
                    &self.media_dir,
                )
            })
            .cloned()
            .collect();

        requested_files
            .iter()
            .filter(|filename| server_file_map.contains_key(*filename))
            .for_each(|filename| {
                cache.insert(filename.clone(), now);
            });

        files_needing_download
    }

    async fn download_files_in_parallel(
        &self,
        filenames: Vec<String>,
        server_file_map: &HashMap<String, ServerMediaFile>,
    ) -> Result<()> {
        download_files_in_parallel(
            &self.source_server,
            &self.media_dir,
            filenames,
            server_file_map,
            self.max_concurrent_downloads,
            Arc::clone(&self.sync_cache),
        )
        .await
    }
}

fn find_missing_files(requested_files: &[String], local_files: &[String]) -> Vec<String> {
    requested_files
        .iter()
        .filter(|file| !local_files.contains(file))
        .cloned()
        .collect()
}

fn should_download_file(
    filename: &String,
    server_file_map: &HashMap<String, ServerMediaFile>,
    cache: &HashMap<String, Instant>,
    now: Instant,
    recent_threshold: Duration,
    media_dir: &Path,
) -> bool {
    let Some(server_file) = server_file_map.get(filename) else {
        debug!("File {} not found on server", filename);
        return false;
    };

    if let Some(&sync_time) = cache.get(filename)
        && now.duration_since(sync_time) < recent_threshold
    {
        debug!("Skipping {} - synced recently", filename);
        return false;
    }

    if is_file_valid(filename, server_file, media_dir) {
        debug!("Skipping {} - already valid", filename);
        false
    } else {
        true
    }
}

pub mod client;
pub mod download;
pub mod validation;
