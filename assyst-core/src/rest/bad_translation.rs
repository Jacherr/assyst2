use std::fmt::Display;

use assyst_common::config::CONFIG;
use reqwest::{Client, Error as ReqwestError};
use serde::Deserialize;
use std::error::Error;

const MAX_ATTEMPTS: u8 = 5;

mod routes {
    pub const LANGUAGES: &str = "/languages";
}

#[derive(Debug)]
pub enum TranslateError {
    Reqwest(ReqwestError),
    Raw(&'static str),
}

impl Display for TranslateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TranslateError::Reqwest(_) => write!(f, "A network error occurred"),
            TranslateError::Raw(s) => write!(f, "{}", s),
        }
    }
}

impl Error for TranslateError {}

#[derive(Deserialize)]
pub struct Translation {
    pub lang: String,
    pub text: String,
}

#[derive(Deserialize)]
pub struct TranslateResult {
    pub translations: Vec<Translation>,
    pub result: Translation,
}

async fn translate_retry(
    client: &Client,
    text: &str,
    target: Option<&str>,
    count: Option<u32>,
    additional_data: Option<&[(&str, String)]>,
) -> Result<TranslateResult, TranslateError> {
    let mut query_args = vec![("text", text.to_owned())];

    if let Some(target) = target {
        query_args.push(("target", target.to_owned()));
    }

    if let Some(count) = count {
        query_args.push(("count", count.to_string()));
    }

    if let Some(data) = additional_data {
        for (k, v) in data.iter() {
            query_args.push((k, v.to_string()));
        }
    }

    client
        .get(&CONFIG.urls.bad_translation)
        .query(&query_args)
        .send()
        .await
        .map_err(TranslateError::Reqwest)?
        .json()
        .await
        .map_err(TranslateError::Reqwest)
}

async fn translate(
    client: &Client,
    text: &str,
    target: Option<&str>,
    count: Option<u32>,
    additional_data: Option<&[(&str, String)]>,
) -> Result<TranslateResult, TranslateError> {
    let mut attempt = 0;

    while attempt <= MAX_ATTEMPTS {
        match translate_retry(client, text, target, count, additional_data).await {
            Ok(result) => return Ok(result),
            Err(e) => eprintln!("Proxy failed! {:?}", e),
        };

        attempt += 1;
    }

    Err(TranslateError::Raw("BT Failed: Too many attempts"))
}

pub async fn bad_translate(client: &Client, text: &str) -> Result<TranslateResult, TranslateError> {
    translate(client, text, None, None, None).await
}

pub async fn bad_translate_with_count(
    client: &Client,
    text: &str,
    count: u32,
) -> Result<TranslateResult, TranslateError> {
    translate(client, text, None, Some(count), None).await
}

pub async fn translate_single(client: &Client, text: &str, target: &str) -> Result<TranslateResult, TranslateError> {
    translate(client, text, Some(target), Some(1), None).await
}

pub async fn get_languages(client: &Client) -> Result<Vec<(Box<str>, Box<str>)>, TranslateError> {
    client
        .get(format!("{}{}", CONFIG.urls.bad_translation, routes::LANGUAGES))
        .send()
        .await
        .map_err(TranslateError::Reqwest)?
        .json()
        .await
        .map_err(TranslateError::Reqwest)
}

// used in btchannel later
#[allow(unused)]
pub async fn validate_language(client: &Client, provided_language: &str) -> Result<bool, TranslateError> {
    let languages = get_languages(client).await?;
    Ok(languages.iter().any(|(language, _)| &**language == provided_language))
}
