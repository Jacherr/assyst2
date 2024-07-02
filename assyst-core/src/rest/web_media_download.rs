use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::bail;
use assyst_common::util::format_duration;
use futures_util::future::join_all;
use rand::seq::SliceRandom;
use rand::thread_rng;
use reqwest::StatusCode;
use serde::Deserialize;
use serde_json::{from_str, json};
use tokio::time::timeout;
use tracing::debug;

use crate::assyst::ThreadSafeAssyst;
use crate::command::flags::DownloadFlags;
use crate::downloader::{download_content, ABSOLUTE_INPUT_FILE_SIZE_LIMIT_BYTES};

pub const INSTANCES_ROUTE: &str = "https://instances.hyper.lol/instances.json";

pub const TEST_URL: &str = "https://www.youtube.com/watch?v=sbvp3kuU2ak";
pub const TEST_SCORE_THRESHOLD: f32 = 90.0;

pub static TEST_URL_TIMEOUT: Duration = Duration::from_secs(15);

#[derive(Default)]
pub struct WebDownloadOpts {
    pub audio_only: Option<bool>,
    pub quality: Option<String>,
    pub api_url_override: Option<String>,
}
impl WebDownloadOpts {
    pub fn from_download_flags(flags: DownloadFlags) -> Self {
        Self {
            audio_only: Some(flags.audio),
            quality: if flags.quality != 0 {
                Some(flags.quality.to_string())
            } else {
                None
            },
            api_url_override: None,
        }
    }
}

#[derive(Deserialize)]
pub struct WebDownloadResult {
    pub url: String,
}

#[derive(Deserialize)]
pub struct WebDownloadError {
    pub text: String,
}

#[derive(Deserialize)]
pub struct InstancesQueryResult {
    pub score: f32,
    pub api: String,
    pub protocol: String,
}

/// Tests a web download route to see if it meets requirements.
/// Requirement is that the entire request finishes in less than 15 seconds on this URL, with a
/// successful download.
/// Returns true if the route is valid, false otherwise.
async fn test_route(assyst: ThreadSafeAssyst, url: &str) -> bool {
    let start = Instant::now();
    let opts = WebDownloadOpts {
        audio_only: Some(false),
        quality: Some("144".to_owned()),
        api_url_override: Some(url.to_owned()),
    };

    let res = download_web_media(assyst.clone(), TEST_URL, opts).await;
    let success = res.is_ok();

    let elapsed = start.elapsed();

    if success && elapsed < TEST_URL_TIMEOUT {
        debug!(
            "Web download URL {url} took {} to download test media",
            format_duration(&elapsed)
        );
    } else if elapsed < TEST_URL_TIMEOUT {
        let err = res.unwrap_err();
        debug!(
            "Web download URL {url} failed to download test media ({})",
            err.to_string()
        );
    }

    success && (elapsed < TEST_URL_TIMEOUT)
}

/// Always returns the main API instance (api.cobalt.tools) at the minimum. \
/// Other URLs must be a score of 100 (i.e., all sites supported) and must have a \
/// domain over https.
pub async fn get_web_download_api_urls(assyst: ThreadSafeAssyst) -> anyhow::Result<Vec<String>> {
    let res = assyst
        .reqwest_client
        .get(INSTANCES_ROUTE)
        .header("accept", "application/json")
        .header("User-Agent", "Assyst Discord Bot (https://github.com/jacherr/assyst2)")
        .send()
        .await?;

    let json = res.json::<Vec<InstancesQueryResult>>().await?;

    let test_urls = json
        .iter()
        .filter_map(|entry: &InstancesQueryResult| {
            if (entry.protocol == "https" && entry.score >= TEST_SCORE_THRESHOLD) || (entry.api == "api.cobalt.tools") {
                Some(format!("https://{}/api/json", entry.api))
            } else {
                None
            }
        })
        .map(|url| {
            debug!("Testing web download API URL {}", url);

            let a = assyst.clone();
            timeout(
                TEST_URL_TIMEOUT,
                tokio::spawn(async move {
                    if url != "https://api.cobalt.tools/api/json" {
                        let res = test_route(a, &url).await;
                        (url, res)
                    } else {
                        (url, true)
                    }
                }),
            )
        })
        .collect::<Vec<_>>();

    let valid_urls = join_all(test_urls)
        .await
        .into_iter()
        .filter_map(|res| res.ok())
        .map(|res| res.unwrap())
        .filter(|res| res.1)
        .map(|res| res.0)
        .collect::<Vec<_>>();

    Ok(valid_urls)
}

/// Attempts to download web media. Will try all APIs in the event of faliure, unless
/// `opts.api_url_override` is set.
pub async fn download_web_media(assyst: ThreadSafeAssyst, url: &str, opts: WebDownloadOpts) -> anyhow::Result<Vec<u8>> {
    let encoded_url = urlencoding::encode(url).to_string();

    let urls = if let Some(api_url) = opts.api_url_override {
        vec![Arc::new(api_url)]
    } else {
        let mut urls = assyst.rest_cache_handler.get_web_download_urls();
        urls.shuffle(&mut thread_rng());
        urls
    };

    let mut req_result_url: Option<String> = None;
    let mut err: String = String::new();

    for route in urls {
        debug!("trying url: {route}");

        let res = assyst
            .reqwest_client
            .post((*route).clone())
            .header("accept", "application/json")
            .header("User-Agent", "Assyst Discord Bot (https://github.com/jacherr/assyst2)")
            .json(&json!({
                "url": encoded_url,
                "isAudioOnly": opts.audio_only.unwrap_or(false),
                "aFormat": "mp3",
                "isNoTTWatermark": true,
                "vQuality": opts.quality.clone().unwrap_or("720".to_owned())
            }))
            .timeout(Duration::from_secs(60))
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
                        Err(e) => err = format!("Failed to deserialize download url: {}", e),
                    }
                } else {
                    let try_err = r.text().await;
                    match try_err {
                        Ok(e) => {
                            let try_json = from_str::<WebDownloadError>(&e);
                            match try_json {
                                Ok(j) => {
                                    let mut e = j.text;
                                    if e.contains("i couldn't process your request :(") {
                                        e = "The web downloader could not process your request. Please try again later."
                                            .to_owned()
                                    } else if e.contains("i couldn't connect to the service api.") {
                                        e = "The web downloader could not connect to the service API. Please try again later.".to_owned()
                                    } else if e.contains("couldn't get this youtube video because it requires sign in")
                                    {
                                        e = "YouTube has blocked video downloading. Please try again later.".to_owned()
                                    }

                                    err = format!("Download request failed: {}", e);
                                },
                                Err(d_e) => err = format!("Download request failed: {} (raw error: {})", d_e, e),
                            }
                        },
                        Err(e) => err = format!("Failed to extract download request error: {}", e),
                    }
                }
            },
            Err(e) => {
                err = format!("Download request failed: {}", e);
            },
        }
    }

    if let Some(r) = req_result_url {
        let media = download_content(&assyst, &r, ABSOLUTE_INPUT_FILE_SIZE_LIMIT_BYTES, false).await?;
        Ok(media)
    } else if !err.is_empty() {
        bail!("{err}");
    } else {
        bail!("Failed to download media: an unknown error occurred");
    }
}
