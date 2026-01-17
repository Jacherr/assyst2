use anyhow::Context;
use assyst_common::config::CONFIG;
use reqwest::{Client, Method};
use serde::Deserialize;
use url::Url;

use crate::downloader::{ABSOLUTE_INPUT_FILE_SIZE_LIMIT_BYTES, download_content};

pub static KLIPY_URL: &'static str = "https://api.klipy.com/api/v1";

#[derive(Deserialize)]
struct KlipyResponse {
    pub result: bool,
    pub data: KlipyData,
}

#[derive(Deserialize)]
struct KlipyData {
    pub data: Vec<KlipyDataInner>,
}

#[derive(Deserialize)]
struct KlipyDataInner {
    pub id: u64,
    pub slug: String,
    pub file: KlipyFile,
}

#[derive(Deserialize)]
struct KlipyFile {
    pub hd: KlipyFileHd,
}

#[derive(Deserialize)]
struct KlipyFileHd {
    gif: KlipyFileGif,
}

#[derive(Deserialize)]
struct KlipyFileGif {
    pub url: String,
}

pub async fn get_klipy_gif_url_from_url(client: &Client, url: &str) -> anyhow::Result<String> {
    let u = Url::parse(&url)?;
    let slug = u.path().split("/").last().unwrap_or("");

    let req_url = format!("{KLIPY_URL}/{}/gifs/items", CONFIG.authentication.klipy_api);
    let res = client
        .request(Method::GET, req_url)
        .query(&[("slugs", slug)])
        .send()
        .await?
        .error_for_status()?
        .json::<KlipyResponse>()
        .await?;

    Ok(res
        .data
        .data
        .first()
        .context("No data returned by klipy")?
        .file
        .hd
        .gif
        .url
        .clone())
}

pub async fn get_klipy_gif_from_klipy_url(client: &Client, url: &str) -> anyhow::Result<Vec<u8>> {
    let res = get_klipy_gif_url_from_url(client, url).await?;

    let content = download_content(client, &res, ABSOLUTE_INPUT_FILE_SIZE_LIMIT_BYTES, true).await?;

    Ok(content)
}
