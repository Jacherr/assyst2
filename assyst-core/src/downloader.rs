use core::fmt;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use assyst_common::config::CONFIG;
use bytes::Bytes;
use futures_util::{Stream, StreamExt};
use human_bytes::human_bytes;
use reqwest::{Client, StatusCode, Url};

pub const ABSOLUTE_INPUT_FILE_SIZE_LIMIT_BYTES: usize = 250_000_000;
static PROXY_NUM: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug)]
pub enum DownloadError {
    ProxyNetworkError,
    InvalidStatus,
    Url(url::ParseError),
    NoHost,
    LimitExceeded(usize),
    Reqwest(reqwest::Error),
}

impl fmt::Display for DownloadError {
    #[rustfmt::skip]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DownloadError::ProxyNetworkError => write!(f, "Failed to connect to proxy"),
            DownloadError::InvalidStatus => write!(f, "Invalid status received from proxy"),
            DownloadError::LimitExceeded(limit) => write!(f, "The output file exceeded the maximum file size limit of {}. Try using a smaller input.", human_bytes((*limit) as f64)),
            DownloadError::Url(e) => write!(f, "Failed to parse URL: {e}"),
            DownloadError::NoHost => write!(f, "No host found in URL"),
            DownloadError::Reqwest(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for DownloadError {}

fn get_next_proxy() -> &'static str {
    let config = &CONFIG;
    let len = config.urls.proxy.len();

    (&config.urls.proxy[PROXY_NUM.fetch_add(1, Ordering::Relaxed) % len]) as _
}

async fn download_with_proxy(
    client: &Client,
    url: &str,
    limit: usize,
) -> Result<impl Stream<Item = Result<Bytes, reqwest::Error>>, DownloadError> {
    let resp = client
        .get(format!("{}/proxy", get_next_proxy()))
        .header("User-Agent", "Assyst Discord Bot (https://github.com/jacherr/assyst2)")
        .query(&[("url", url), ("limit", &limit.to_string())])
        .timeout(Duration::from_secs(10))
        .send()
        .await
        .map_err(|_| DownloadError::ProxyNetworkError)?;

    if resp.status() != StatusCode::OK {
        return Err(DownloadError::InvalidStatus);
    }

    Ok(resp.bytes_stream())
}

async fn download_no_proxy(
    client: &Client,
    url: &str,
) -> Result<impl Stream<Item = Result<Bytes, reqwest::Error>>, DownloadError> {
    Ok(client
        .get(url)
        .header("User-Agent", "Assyst Discord Bot (https://github.com/jacherr/assyst2)")
        .send()
        .await
        .map_err(DownloadError::Reqwest)?
        .bytes_stream())
}

async fn read_stream<S>(mut stream: S, limit: usize) -> Result<Vec<u8>, DownloadError>
where
    S: Stream<Item = Result<Bytes, reqwest::Error>> + Unpin,
{
    let mut bytes = Vec::new();

    while let Some(Ok(chunk)) = stream.next().await {
        if bytes.len() > limit {
            return Err(DownloadError::LimitExceeded(limit));
        }

        bytes.extend(chunk);
    }

    Ok(bytes)
}

/// Attempts to download a resource from a URL.
pub async fn download_content(
    client: &Client,
    url: &str,
    limit: usize,
    untrusted: bool,
) -> Result<Vec<u8>, DownloadError> {
    const WHITELISTED_DOMAINS: &[&str] = &[
        "tenor.com",
        "jacher.io",
        "discordapp.com",
        "discordapp.net",
        "wuk.sh",
        "gyazo.com",
        "cdn.discordapp.com",
        "media.discordapp.net",
        "notsobot.com",
        "twimg.com",
        "cdninstagram.com",
        "imput.net",
    ];

    let config = &CONFIG;

    let url_p = Url::parse(url).map_err(DownloadError::Url)?;
    let host = url_p.host_str().ok_or(DownloadError::NoHost)?;

    let is_whitelisted = WHITELISTED_DOMAINS.iter().any(|d| host.contains(d));

    if !config.urls.proxy.is_empty() && !is_whitelisted && untrusted {
        // First, try to download with proxy
        let stream = download_with_proxy(client, url, limit).await;

        if let Ok(stream) = stream {
            return read_stream(stream, limit).await;
        }
    }

    // Conditions for downloading with no proxy:
    // - Proxy not configured,
    // - Proxy failed,
    // - Domain is whitelisted
    let stream = download_no_proxy(client, url).await?;
    read_stream(stream, limit).await
}
