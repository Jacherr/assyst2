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

pub async fn cooltext(style: Style, text: &str) -> anyhow::Result<Vec<u8>> {
    let client = ClientBuilder::new().danger_accept_invalid_certs(true).build().unwrap();

    let cool_text_response = client
        .post(COOLTEXT_URL)
        .query(query_for_style(style, text, "70").as_slice())
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

// generates a url query for a given style, using the default parameters
fn query_for_style<'a>(style: Style, text: &'a str, font_size: &'a str) -> Vec<(&'a str, &'a str)> {
    let mut v = vec![
        ("Text", text),
        ("FontSize", font_size),
        ("FileFormat", "6"),                  // .PNG w/ Transparency
        ("Integer13", "on"),                  // auto width
        ("Integer12", "on"),                  // auto height
        ("BackgroundColor_color", "#FFFFFF"), // white background
    ];
    v.extend(match style {
        Style::Skate => vec![
            ("LogoID", "4610356863"),
            ("Color1_color", "#83D6FC"),
            ("Color2_color", "#0088FF"),
            ("Color3_color", "#000000"),
            ("Integer1", "5"),
            ("Integer5", "2"),
            ("Integer7", "1"),
            ("Integer8", "1"),
            ("Integer14_color", "#000000"),
            ("Integer6", "75"),
            ("Integer9", "0"),
        ],
        Style::SuperScripted => vec![
            ("LogoID", "4610363770"),
            ("Color1_color", "#000000"),
            ("Integer5", "4"),
            ("Integer7", "0"),
            ("Integer8", "0"),
            ("Integer14_color", "#000000"),
            ("Integer6", "70"),
            ("Integer9", "0"),
        ],
        Style::Tough => vec![
            ("LogoID", "758282876"),
            ("Color1_color", "#0A213D"),
            ("Integer1", "5"),
            ("Integer5", "0"),
            ("Integer7", "0"),
            ("Integer8", "0"),
            ("Integer14_color", "#000000"),
            ("Integer6", "75"),
            ("Integer9", "0"),
        ],
        Style::White => vec![
            ("LogoID", "4610365972"),
            ("Color1_color", "#000000"),
            ("Color2_color", "#FFFFFF"),
            ("Color3_color", "#FFFFFF"),
            ("Integer5", "0"),
            ("Integer7", "0"),
            ("Integer8", "0"),
            ("Integer14_color", "#000000"),
            ("Integer6", "75"),
            ("Integer9", "0"),
        ],
        Style::IText => vec![
            ("LogoID", "37"),
            ("Color1_color", "#669900"),
            ("Integer5", "0"),
            ("Integer7", "0"),
            ("Integer8", "0"),
            ("Integer14_color", "#000000"),
            ("Integer6", "75"),
            ("Integer9", "0"),
        ],
        Style::Easy => vec![
            ("LogoID", "791030843"),
            ("Color1_color", "#004A99"),
            ("Integer5", "4"),
            ("Integer7", "0"),
            ("Integer8", "0"),
            ("Integer14_color", "#000000"),
            ("Integer6", "60"),
            ("Integer9", "0"),
        ],
        Style::Textured => vec![
            ("LogoID", "23"),
            ("Color2_color", "#206A00"),
            ("Color3_color", "#00006A"),
            ("Integer9", "0"),
        ],
        Style::Stranger => vec![
            ("LogoID", "4610366723"),
            ("Color1_color", "#F5FAFF"),
            ("Color2_color", "#16877A"),
            ("Color3_color", "#000000"),
            ("Integer1", "5"),
            ("Integer5", "2"),
            ("Integer7", "1"),
            ("Integer8", "1"),
            ("Integer14_color", "#000000"),
            ("Integer6", "75"),
            ("Integer9", "0"),
        ],
        Style::Burning => vec![
            ("LogoID", "4"),
            ("Color1_color", "#FF0000"),
            ("Integer1", "15"),
            ("Boolean1", "on"),
            ("Integer9", "0"),
        ],
        Style::Neon => vec![
            ("LogoID", "18"),
            ("Color1_color", "#23D3FF"),
            ("Integer5", "1"),
            ("Integer7", "3"),
            ("Integer8", "3"),
            ("Integer14_color", "#000000"),
            ("Integer6", "85"),
            ("Integer9", "0"),
        ],
        Style::Candy => vec![
            ("LogoID", "732431452"),
            ("Color1_color", "#FF00FF"),
            ("Integer5", "2"),
            ("Integer7", "2"),
            ("Integer8", "2"),
            ("Integer14_color", "#000000"),
            ("Integer6", "25"),
            ("Integer9", "0"),
        ],
        Style::Saint => vec![("LogoID", "4516516448")],
    });
    v
}

pub enum Style {
    Skate,
    SuperScripted,
    Tough,
    White,
    IText,
    Easy,
    Textured,
    Stranger,
    Burning,
    Neon,
    Candy,
    Saint,
}

// this sucks, replace with some proc macro that does the following 3 impls for free
impl Style {
    pub fn list() -> &'static [Self] {
        &[
            Self::Skate,
            Self::SuperScripted,
            Self::Tough,
            Self::White,
            Self::IText,
            Self::Easy,
            Self::Textured,
            Self::Stranger,
            Self::Burning,
            Self::Neon,
            Self::Candy,
            Self::Saint,
        ]
    }
}

impl Display for Style {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Skate => "skate",
                Self::SuperScripted => "super_scripted",
                Self::Tough => "tough",
                Self::White => "white",
                Self::IText => "itext",
                Self::Easy => "easy",
                Self::Textured => "textured",
                Self::Stranger => "stranger",
                Self::Burning => "burning",
                Self::Neon => "neon",
                Self::Candy => "candy",
                Self::Saint => "saint",
            }
        )
    }
}

impl FromStr for Style {
    type Err = &'static str; // FIXME: this should not be a string.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "skate" => Ok(Self::Skate),
            "super_scripted" => Ok(Self::SuperScripted),
            "tough" => Ok(Self::Tough),
            "white" => Ok(Self::White),
            "itext" => Ok(Self::IText),
            "easy" => Ok(Self::Easy),
            "textured" => Ok(Self::Textured),
            "stranger" => Ok(Self::Stranger),
            "burning" => Ok(Self::Burning),
            "neon" => Ok(Self::Neon),
            "candy" => Ok(Self::Candy),
            "saint" => Ok(Self::Saint),
            _ => Err("unknown style"),
        }
    }
}
