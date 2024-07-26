use anyhow::Context;
use assyst_common::config::CONFIG;
use serde::Deserialize;

use crate::assyst::ThreadSafeAssyst;

#[derive(Deserialize)]
pub struct FilerStats {
    pub count: u64,
    pub size_bytes: u64,
}

pub async fn get_filer_stats(assyst: ThreadSafeAssyst) -> anyhow::Result<FilerStats> {
    Ok(assyst.reqwest_client.get(&format!("{}/stats", CONFIG.urls.filer)).send().await?.json::<FilerStats>().await?)
}

pub async fn upload_to_filer(assyst: ThreadSafeAssyst, data: Vec<u8>, content_type: &str) -> anyhow::Result<String> {
    Ok(assyst
        .reqwest_client
        .post(&CONFIG.urls.filer)
        .header(reqwest::header::CONTENT_TYPE, content_type)
        .header(reqwest::header::AUTHORIZATION, &CONFIG.authentication.filer_key)
        .body(data)
        .send()
        .await?
        .error_for_status()
        .context("Failed to upload to filer")?
        .text()
        .await?)
}
