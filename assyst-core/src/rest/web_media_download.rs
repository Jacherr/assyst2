use std::fmt::Display;
use std::time::Duration;

use anyhow::{Context, bail};
use assyst_common::config::config::CobaltApiInstance;
use assyst_common::util::string_from_likely_utf8;
use rand::seq::SliceRandom;
use rand::thread_rng;
use reqwest::{Client, StatusCode};
use serde::Deserialize;
use serde_json::{from_str, json};
use tokio::process::Command;
use tokio::time::timeout;
use tracing::{debug, info};

use crate::command::services::download::DownloadFlags;
use crate::downloader::{ABSOLUTE_INPUT_FILE_SIZE_LIMIT_BYTES, download_content};

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
    pub error: WebDownloadErrorContext,
}
impl Display for WebDownloadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let inner = &self.error.code;
        match &inner.to_ascii_lowercase()[..] {
            "error.api.unreachable" => f.write_str("API unreachable (try again later)"),
            "error.api.timed_out" => f.write_str("API timeout (try again later)"),
            "error.api.rate_exceeded" => f.write_str("Rate limited (try again later)"),
            "error.api.capacity" => f.write_str("API busy (try again later)"),
            "error.api.generic" => f.write_str("General API error (try again later)"),
            "error.api.unknown_response" => {
                f.write_str("Download failure. Make sure the link is valid. (unknown response)")
            },
            "error.api.service.unsupported" => f.write_str("That service or website is not supported."),
            "error.api.service.disabled" => {
                f.write_str("Downloading from that service or website is temporarily disabled.")
            },
            "error.api.link.invalid" => f.write_str("That link is invalid. Make sure it is correct."),
            "error.api.link.unsupported" => f.write_str("That link or format is unsupported."),
            "error.api.fetch.fail" => f.write_str("Failed to fetch the media. Make sure the link is valid, or try again later."),
            "error.api.fetch.critical" => f.write_str("Critical error fetching the media. Make sure the link is valid, or try again later."),
            "error.api.fetch.empty" => f.write_str("The service or website returned no data. This may be caused by the site blocking the downloader (try again later)"),
            "error.api.fetch.rate" => f.write_str("The service or website has rate limited the downloader (try again later)"),
            "error.api.fetch.short_link" => f.write_str("Unable to resolve the shortlink. Try using the full link to the media."),
            "error.api.content.too_long" => f.write_str("The requested content is too big."),
            "error.api.content.video.unavailable" => f.write_str(
                "That video is unavailable. Make sure it is not region or age restricted, and is not private.",
            ),
            "error.api.content.video.live" => f.write_str("Live videos are unsupported."),
            "error.api.content.video.private" => f.write_str("That video is private."),
            "error.api.content.video.age" => f.write_str("That video is age restricted."),
            "error.api.content.video.region" => f.write_str("That video is region restricted."),
            "error.api.content.post.unavailable" => f.write_str(
                "That post is unavailable. Make sure it is not region or age restricted, and is not private.",
            ),
            "error.api.content.post.private" => f.write_str("That post is private."),
            "error.api.content.post.age" => f.write_str("That post is age restricted."),
            "error.api.youtube.codec" => f.write_str("Missing YouTube codec. This is a bug."),
            "error.api.youtube.decipher" => f.write_str("Cannot decipher that video. Something probably broke."),
            "error.api.youtube.login" => f.write_str("That video requires a logged in account, which we do not have."),
            "error.api.youtube.token_expired" => f.write_str("Our YouTube token expired (try again later)"),
            "error.api.youtube.temporary_disabled" => f.write_str("YouTube support is temporarily disabled. Try again later."),
            "error.api.auth.key.ip_not_allowed" => f.write_str("The request was blocked. Try again later."),
            _ => f.write_str(inner),
        }
    }
}

#[derive(Deserialize)]
pub struct WebDownloadErrorContext {
    code: String,
}

/// Attempts to download web media. Will try all APIs until one succeeds, unless
/// `opts.api_url_override` is set.
pub async fn download_web_media(
    client: &Client,
    url: &str,
    opts: WebDownloadOpts,
) -> anyhow::Result<(Vec<u8>, String)> {
    let urls = {
        let mut urls = opts.urls;
        if urls.is_empty() {
            bail!("No available instances are defined.");
        }
        urls.shuffle(&mut thread_rng());
        // put primary ones first
        urls.sort_by(|a, b| b.primary.unwrap_or(false).cmp(&a.primary.unwrap_or(false)));
        urls
    };

    let mut result: Option<Vec<u8>> = None;
    let mut err: String = String::new();
    let mut u = String::new();

    for route in urls {
        let key = route.key.clone();
        let route = route.url.clone();

        debug!("trying url: {route} for web media {url} (err={err})");

        let res = client
            .post(route.clone())
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
                                    err = format!("Download request failed: {j}");
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

        if !err.is_empty() && !err.ends_with(")") {
            err = format!("{err} (Instance: {})", route.clone())
        }

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
                    err = format!("Failed to download media: {e} (Instance: {})", route.clone());
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
                err = format!(
                    "Failed to download media: response was: {s} (Instance: {})",
                    route.clone()
                );
                continue;
            } else if media.is_empty() {
                err = "Failed to download media: resultant file was empty".to_owned();
                continue;
            }

            result = Some(media);
            u = route.clone();
            break;
        }
    }

    if let Some(r) = result {
        Ok((r, u.to_string()))
    } else {
        bail!(err)
    }
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
