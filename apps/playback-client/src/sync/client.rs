use crate::sync::ServerMediaFile;
use crate::sync::ServerMediaList;
use anyhow::{bail, Context, Result};
use std::time::Duration;
use tracing::debug;

pub struct MediaClient {
    client: reqwest::Client,
    source_server: String,
}

impl MediaClient {
    pub fn new(source_server: &str, timeout_seconds: u64) -> Result<Self> {
        let timeout = Duration::from_secs(timeout_seconds);
        let client = reqwest::Client::builder()
            .timeout(timeout)
            .pool_max_idle_per_host(1)
            .build()
            .context("failed to create HTTP client for media sync")?;

        Ok(Self {
            client,
            source_server: source_server.to_string(),
        })
    }

    pub async fn fetch_server_media_list(&self) -> Result<Vec<ServerMediaFile>> {
        let url = format!("{}/api/media", self.source_server);

        let response = self.client.get(&url).send().await?;

        if response.status().as_u16() != 200 {
            bail!("Server returned status {}", response.status());
        }

        let media_list: ServerMediaList = response.json().await?;

        if !media_list.success {
            bail!("Server returned success=false");
        }

        debug!(
            "Server media list retrieved: {} files",
            media_list.data.len()
        );
        Ok(media_list.data)
    }
}
