use std::hash::{DefaultHasher, Hash, Hasher};

use anyhow::bail;
use reqwest::ClientBuilder;
use serde::Deserialize;

static COOLTEXT_URL: &str = "https://cooltext.com/PostChange";

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoolTextResponse {
    pub render_location: String,
}

pub async fn burn_text(text: &str) -> anyhow::Result<Vec<u8>> {
    let client = ClientBuilder::new().danger_accept_invalid_certs(true).build().unwrap();

    // Don't ask what most of these parameters do, because I don't know.
    // FIXME: Find out which of these query params are actually necessary
    let cool_text_response = client
        .post(COOLTEXT_URL)
        .query(&[
            ("LogoID", "4"),
            ("Text", text),
            ("FontSize", "70"),
            ("Color1_color", "#FF0000"),
            ("Integer1", "15"),
            ("Boolean1", "on"),
            ("Integer9", "0"),
            ("Integer13", "on"),
            ("Integer12", "on"),
            ("BackgroundColor_color", "#FFFFFF"),
        ])
        .header("content-length", "0")
        .send()
        .await?
        .json::<CoolTextResponse>()
        .await?;

    let url = cool_text_response.render_location;
    let content = client.get(&url.replace("https", "http")).send().await?.bytes().await?;

    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    let result = hasher.finish();

    if result == 3837314301372762351
    /* image deleted/invalid etc */
    {
        bail!("failed to process input, most likely it's too long or contains invalid characters")
    }

    Ok(content.to_vec())
}
