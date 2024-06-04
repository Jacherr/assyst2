use anyhow::bail;
use reqwest::StatusCode;
use serde::Deserialize;
use serde_json::{from_str, json};

use crate::assyst::ThreadSafeAssyst;
use crate::downloader::{download_content, ABSOLUTE_INPUT_FILE_SIZE_LIMIT_BYTES};

/// Default to main route (aka index 0). Use fallback routes in order on failure.
pub const ROUTES: &[&str] = &[
    "https://api.cobalt.tools/api/json",
    "https://api-dl.cgm.rs/api/json",
    "https://cobapi.fly.dev/api/json",
];

#[derive(Default)]
pub struct WebDownloadOpts {
    pub audio_only: Option<bool>,
    pub quality: Option<String>,
}

#[derive(Deserialize)]
pub struct WebDownloadResult {
    pub url: String,
}

#[derive(Deserialize)]
pub struct WebDownloadError {
    pub text: String,
}

pub async fn download_web_media(assyst: ThreadSafeAssyst, url: &str, opts: WebDownloadOpts) -> anyhow::Result<Vec<u8>> {
    let encoded_url = urlencoding::encode(url).to_string();

    let mut req_result_url: Option<String> = None;
    let mut err: String = String::new();

    for route in ROUTES {
        let res = assyst
            .reqwest_client
            .post(*route)
            .header("accept", "application/json")
            .header("User-Agent", "Assyst Discord Bot (https://github.com/jacherr/assyst2)")
            .json(&json!({
                "url": encoded_url,
                "isAudioOnly": opts.audio_only.unwrap_or(false),
                "aFormat": "mp3",
                "isNoTTWatermark": true,
                "vQuality": opts.quality.clone().unwrap_or("720".to_owned())
            }))
            .send()
            .await;

        match res {
            Ok(r) => {
                if r.status() == StatusCode::OK {
                    let try_json = r.json::<WebDownloadResult>().await;
                    match try_json {
                        Ok(j) => {
                            req_result_url = Some(j.url.to_string());
                            break;
                        },
                        Err(e) => err = format!("Failed to deserialize download url: {}", e.to_string()),
                    }
                } else {
                    let try_err = r.text().await;
                    match try_err {
                        Ok(e) => {
                            let try_json = from_str::<WebDownloadError>(&e);
                            match try_json {
                                Ok(j) => err = format!("Download request failed: {}", j.text),
                                Err(d_e) => {
                                    err = format!("Download request failed: {} (raw error: {})", d_e.to_string(), e)
                                },
                            }
                        },
                        Err(e) => err = format!("Failed to extract download request error: {}", e.to_string()),
                    }
                }
            },
            Err(e) => {
                let mut e = e.to_string();
                if e.contains("i couldn't process your request :(") {
                    e = "The web downloader could not process your request. Please try again later.".to_owned()
                } else if e.contains("i couldn't connect to the service api.") {
                    e = "The web downloader could not connect to the service API. Please try again later.".to_owned()
                }

                err = format!("Download request failed: {}", e.to_string());
            },
        }
    }

    if let Some(r) = req_result_url {
        let media = download_content(&assyst, &r, ABSOLUTE_INPUT_FILE_SIZE_LIMIT_BYTES, false).await?;
        return Ok(media);
    } else if !err.is_empty() {
        bail!("{err}");
    } else {
        bail!("Failed to download media: an unknown error occurred");
    }
}
