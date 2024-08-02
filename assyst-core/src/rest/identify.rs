use assyst_common::config::CONFIG;
use reqwest::Client;
use serde::{Deserialize, Serialize};

const IDENTIFY_ROUTE: &str = "https://microsoft-computer-vision3.p.rapidapi.com/analyze?language=en&descriptionExclude=Celebrities&visualFeatures=Description&details=Celebrities";

#[derive(Serialize)]
pub struct IdentifyBody<'a> {
    pub url: &'a str,
}

#[derive(Deserialize)]
pub struct IdentifyResponse {
    pub description: Option<IdentifyDescription>,
}

#[derive(Deserialize)]
pub struct IdentifyDescription {
    pub captions: Vec<IdentifyCaption>,
}

#[derive(Deserialize)]
pub struct IdentifyCaption {
    pub text: String,
    pub confidence: f32,
}

pub async fn identify_image(client: &Client, url: &str) -> reqwest::Result<IdentifyResponse> {
    client
        .post(IDENTIFY_ROUTE)
        .header("x-rapidapi-key", &CONFIG.authentication.rapidapi_token)
        .json(&IdentifyBody { url })
        .send()
        .await?
        .json()
        .await
}
