use std::time::Duration;

use anyhow::{bail, Context};
use assyst_common::config::config::CobaltApiInstance;
use assyst_common::util::string_from_likely_utf8;
use rand::seq::SliceRandom;
use rand::thread_rng;
use reqwest::{Client, StatusCode};
use serde::Deserialize;
use serde_json::{from_str, json};
use tokio::process::Command;
use tokio::time::timeout;
use tracing::debug;

use crate::command::services::download::DownloadFlags;
use crate::downloader::{download_content, ABSOLUTE_INPUT_FILE_SIZE_LIMIT_BYTES};

pub const INSTANCES_ROUTE: &str = "https://instances.hyper.lol/instances.json";

pub const TEST_URL: &str = "https://www.youtube.com/watch?v=sbvp3kuU2ak";
pub const TEST_SCORE_THRESHOLD: f32 = 90.0;

pub static TEST_URL_TIMEOUT: Duration = Duration::from_secs(15);

#[derive(Default, Clone)]
pub struct WebDownloadOpts {
    pub audio_only: Option<bool>,
    pub quality: Option<String>,
    pub urls: Vec<CobaltApiInstance>,
    pub verbose: bool,
}
impl WebDownloadOpts {
    pub fn from_download_flags(flags: DownloadFlags, urls: Vec<CobaltApiInstance>) -> Self {
        Self {
            audio_only: Some(flags.audio),
            quality: if flags.quality != 0 {
                Some(flags.quality.to_string())
            } else {
                None
            },
            urls,
            verbose: flags.verbose,
        }
    }
}

#[derive(Deserialize)]
pub struct YouTubePlaylist {
    pub entries: Vec<YouTubePlaylistEntry>,
}

#[derive(Deserialize)]
pub struct YouTubePlaylistEntry {
    pub url: String,
    pub title: String,
    pub duration: Option<f32>,
}

#[derive(Deserialize)]
pub struct WebDownloadResult {
    pub url: String,
    #[serde(rename = "status")]
    pub _status: String,
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

/// Attempts to download web media. Will try all APIs until one succeeds, unless
/// `opts.api_url_override` is set.
pub async fn download_web_media(client: &Client, url: &str, opts: WebDownloadOpts) -> anyhow::Result<Vec<u8>> {
    let urls = {
        let mut urls = opts.urls;
        if urls.is_empty() {
            bail!("No available instances are defined.");
        }
        urls.shuffle(&mut thread_rng());
        urls
    };

    let mut result: Option<Vec<u8>> = None;
    let mut err: String = String::new();

    for route in urls {
        let key = route.key.clone();
        let route = route.url.clone();

        debug!("trying url: {route} for web media {url}");

        let res = client
            .post(route)
            .header("accept", "application/json")
            .header("content-type", "application/json")
            .header("User-Agent", "Assyst Discord Bot (https://github.com/jacherr/assyst2)")
            .header("Authorization", key)
            .json(&json!({
                "url": url,
                "downloadMode": if opts.audio_only.unwrap_or(false) { "audio" } else { "auto" },
                "audioFormat": "mp3",
                "videoQuality": opts.quality.clone().unwrap_or("720".to_owned()),
            }))
            .timeout(Duration::from_secs(60))
            .send()
            .await;

        let mut req_result_url = None;
        match res {
            Ok(r) => {
                if r.status() == StatusCode::OK {
                    let try_json = r.json::<WebDownloadResult>().await;
                    match try_json {
                        Ok(j) => {
                            req_result_url = Some(j.url.to_string());
                        },
                        Err(e) => err = format!("Failed to deserialize download url: {e}"),
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

                                    err = format!("Download request failed: {e}");
                                },
                                Err(d_e) => err = format!("Download request failed: {d_e} (raw error: {e})"),
                            }
                        },
                        Err(e) => err = format!("Failed to extract download request error: {e}"),
                    }
                }
            },
            Err(e) => {
                err = format!("Download request failed: {e}");
            },
        };

        if let Some(r) = req_result_url {
            debug!("downloading from url {r} for web media {url}");

            let media = match timeout(
                Duration::from_secs(120),
                download_content(client, &r, ABSOLUTE_INPUT_FILE_SIZE_LIMIT_BYTES, false),
            )
            .await
            {
                Ok(Ok(m)) => m,
                Ok(Err(e)) => {
                    err = format!("Failed to download media: {e}");
                    continue;
                },
                Err(_) => {
                    err = "Failed to download media: a timeout was exceeded".to_owned();
                    continue;
                },
            };

            if let Ok(s) = String::from_utf8(media.clone())
                && s.starts_with("<!DOCTYPE")
            {
                err = "Failed to download media: cloudlflare threw an error".to_owned();
                continue;
            } else if let Ok(s) = String::from_utf8(media.clone()) {
                err = format!("Failed to download media: response was: {s}");
                continue;
            } else if media.is_empty() {
                err = "Failed to download media: resultant file was empty".to_owned();
                continue;
            }

            result = Some(media);
            break;
        }
    }

    if let Some(r) = result { Ok(r) } else { bail!(err) }
}

pub async fn get_youtube_playlist_entries(url: &str) -> anyhow::Result<Vec<(String, String)>> {
    let mut command = Command::new("yt-dlp");
    command.args(["--flat-playlist", "--no-warnings", "-q", "-i", "-J", url]);
    let result = command.output().await.context("Failed to get playlist entries")?;
    if !result.status.success() {
        bail!(
            "Failed to get playlist entries: {}",
            string_from_likely_utf8(result.stderr)
        );
    }

    let output = string_from_likely_utf8(result.stdout);
    let playlist = from_str::<YouTubePlaylist>(&output).context("Failed to deserialize playlist")?;

    // longest videos first
    let mut entries = playlist.entries;
    entries.sort_by(|x, y| y.duration.unwrap_or(0.0).total_cmp(&x.duration.unwrap_or(0.0)));

    Ok(entries
        .iter()
        .map(|x| (x.title.clone(), x.url.clone()))
        .collect::<Vec<_>>())
}
