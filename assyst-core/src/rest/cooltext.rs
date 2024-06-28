use std::fmt::Display;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::str::FromStr;

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
            ("LogoID", "4"), // determines that this is the 'Burning' text
            ("Text", text),
            ("FontSize", "70"),
            ("Color1_color", "#FF0000"),
            ("Integer1", "15"), // angle the flames are rendered at, 0-360
            ("Boolean1", "on"), // transparency
            ("Integer9", "0"),  /* alignment, number is one of
                                   Top Left (0),    Top Center (1),    Top Right (2),
                                Middle Left (3),      Centered (4), Middle Right (5),
                                Bottom Left (6), Bottom Center (7), Bottom Right (8), */
            ("Integer13", "on"), // width of the image, "on" for auto
            ("Integer12", "on"), // height of the image, "on" for auto
            ("BackgroundColor_color", "#FFFFFF"),
        ])
        .header("content-length", "0")
        .send()
        .await?
        .json::<CoolTextResponse>()
        .await?;

    let url = cool_text_response.render_location;
    let content = client.get(url.replace("https", "http")).send().await?.bytes().await?;

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

pub async fn cooltext(style: &str, text: &str) -> anyhow::Result<Vec<u8>> {
    let client = ClientBuilder::new().danger_accept_invalid_certs(true).build().unwrap();
    let styled = STYLES
        .iter()
        .find_map(|(x, y)| if *x == style { Some(y) } else { None })
        .ok_or(anyhow::anyhow!("unknown style {style}"))?;

    let cool_text_response = client
        .post(COOLTEXT_URL)
        .query(&[
            ("LogoID", *styled),
            ("Text", text),
            ("FontSize", "70"),
            ("FileFormat", "6"),
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
    let content = client.get(url.replace("https", "http")).send().await?.bytes().await?;

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

pub const STYLES: &[(&str, &str)] = &[
    ("skate", "4610356863"),
    ("super_scripted", "4610363770"),
    ("tough", "758282876"),
    ("white", "4610365972"),
    ("itext", "37"),
    ("easy", "791030843"),
    ("textured", "23"),
    ("stranger", "4610366723"),
    ("burning", "4"),
    ("neon", "18"),
    ("candy", "732431452"),
    ("saint", "4516516448"),
    ("3d_outline", "4611483973"),
];
