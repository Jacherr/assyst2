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

    if let Ok(a) = String::from_utf8(content.to_vec())
        && a.starts_with("<!DOCTYPE")
    {
        bail!("Cooltext failed to process your input. Try a different input.")
    }

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
        .find_map(|(x, y)| if x.to_lowercase() == style.to_lowercase() { Some(y) } else { None })
        .ok_or(anyhow::anyhow!("Unknown style {style}. Try the 'list' subcommand to see all available styles."))?;

    let cool_text_response = client
        .post(COOLTEXT_URL)
        .query(&[
            ("LogoID", *styled),
            ("Text", text),
            ("FontSize", "70"),
            ("FileFormat", "6"),
            ("Integer13", "on"),
            ("Integer12", "on"),
            ("Boolean1", "on"),
            ("BackgroundColor_color", "#FFFFFF"),
        ])
        .header("content-length", "0")
        .send()
        .await?
        .json::<CoolTextResponse>()
        .await?;

    let url = cool_text_response.render_location;
    let content = client.get(url.replace("https", "http")).send().await?.bytes().await?;

    if let Ok(a) = String::from_utf8(content.to_vec())
        && a.starts_with("<!DOCTYPE")
    {
        bail!("Cooltext failed to process your input. Try a different input.")
    }

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
    ("3D_Outline_Gradient", "29"),
    ("3D_Outline_Textured", "28"),
    ("Alien_Glow", "1"),
    ("Animated_Glow", "26"),
    ("Apollo_11", "4113153856"),
    ("Astroman", "4112238638"),
    ("Bad_Acid", "732450628"),
    ("Black_Gold", "4516496663"),
    ("Black_Hole", "602543131"),
    ("Blended", "2"),
    ("Blinkie", "819515844"),
    ("Bovinated", "3"),
    ("Burning", "4"),
    ("Candy", "732431452"),
    ("Candy_Stripe", "1783676518"),
    ("Carved", "5"),
    ("Caster", "2654162149"),
    ("Chalk", "6"),
    ("Cheetah", "783763343"),
    ("Chick_Flick", "622063540"),
    ("Chrome_One", "7"),
    ("Chrome_Two", "8"),
    ("Chromium", "33"),
    ("Club", "832337804"),
    ("Coffee_Cup", "4528246004"),
    ("Comic", "9"),
    ("Cool_Metal", "10"),
    ("Crystal", "11"),
    ("Cupid", "622058564"),
    ("Cutout", "12"),
    ("Dark", "830474754"),
    ("Dark_Magic", "2975674466"),
    ("DEEJAY", "4112285956"),
    ("Dragon", "1408867449"),
    ("Easy", "791030843"),
    ("Embossed", "31"),
    ("Epic_Stone", "732440996"),
    ("Fantasy", "45"),
    ("Felt", "24"),
    ("Fire", "13"),
    ("Flaming", "1169711118"),
    ("Frosty", "36"),
    ("Fun", "1009848424"),
    ("Galactica", "599808801"),
    ("Gelatin", "4516518098"),
    ("Glitter", "44"),
    ("Glossy", "30"),
    ("Glowing_Hot", "14"),
    ("Glowing_Steel", "15"),
    ("Gold_Bar", "757794054"),
    ("GOLD_BEVEL", "4112424040"),
    ("Gold_Outline", "46"),
    ("Gold_Trim", "732443655"),
    ("Gradient_Bevel", "16"),
    ("Graffiti_Creator", "4110593891"),
    ("Greek_Gold", "4112421173"),
    ("Grinch", "1516206867"),
    ("Groovy", "789574607"),
    ("Grunge", "35"),
    ("Gunmetal", "852819205"),
    ("Halloween", "1408818473"),
    ("Happy_Joy", "1516212090"),
    ("Happy_New_Year", "2222569522"),
    ("Hot", "833904313"),
    ("Hot_Pink", "2651216203"),
    ("House_Arryn", "783758829"),
    ("Ice_Cube", "1779834160"),
    ("Iceberg", "783756759"),
    ("Iced", "34"),
    ("Imprint", "615602790"),
    ("iText", "37"),
    ("Keen", "758279718"),
    ("Klingon", "599808502"),
    ("Lasers", "611409107"),
    ("Lava", "852774362"),
    ("Legal", "732429307"),
    ("Liquid_Gold", "1279361064"),
    ("Love", "819721038"),
    ("Merry_Christmas", "2222568262"),
    ("Miami", "2854656927"),
    ("Molten_Core", "43"),
    ("Muddy", "615608693"),
    ("Neon", "18"),
    ("Neron", "2176267473"),
    ("Nova", "19"),
    ("Old_Stone", "27"),
    ("Orange", "943456044"),
    ("Outline", "25"),
    ("Particle", "39"),
    ("Pixel_Badge", "32"),
    ("Plain", "4112952183"),
    ("Plastic", "42"),
    ("Popsicle", "615600713"),
    ("Princess", "829964308"),
    ("Quicksilver", "790967832"),
    ("Rage", "749791093"),
    ("Robot", "2655372531"),
    ("Romance", "4112260450"),
    ("Royal", "1357657967"),
    ("SAINT", "4516516448"),
    ("Saint_Patrick", "758277984"),
    ("Scavenge", "4110551533"),
    ("Serial", "742083872"),
    ("Simple", "21"),
    ("Skate", "780833150"),
    ("Slab", "17"),
    ("Snowman", "615569527"),
    ("Spaced_Out", "2655376160"),
    ("Spring", "759902224"),
    ("Starburst", "22"),
    ("Stranger", "2792545512"),
    ("Studio_54", "732453157"),
    ("Sugar", "1783669883"),
    ("Super_Scripted", "732447945"),
    ("Supernova", "2650967346"),
    ("Sushi", "830446526"),
    ("Sword", "2172004512"),
    ("Tesla", "4113131447"),
    ("Textured", "23"),
    ("Tie_Dyed", "612444173"),
    ("Tough", "758282876"),
    ("Tribal", "2975689126"),
    ("Trogdor", "4112270507"),
    ("Vampire", "732414977"),
    ("Warp", "599825692"),
    ("Water", "830469381"),
    ("White", "732438332"),
    ("Will_You_Marry_Me", "4112242098"),
    ("Wizards", "38"),
];
