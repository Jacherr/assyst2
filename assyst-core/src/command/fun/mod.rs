use std::time::Duration;

use anyhow::Context;
use assyst_proc_macro::command;

use super::arguments::ImageUrl;
use super::CommandCtxt;
use crate::command::{Availability, Category};
use crate::rest::audio_identification::identify_song_notsoidentify;

pub mod colour;
pub mod translation;

#[command(
    description = "Find a song in a video",
    cooldown = Duration::from_secs(2),
    access = Availability::Public,
    category = Category::Fun,
    usage = "[video]",
    examples = ["https://link.to.my/video.mp4"],
    send_processing = true
)]
pub async fn findsong(ctxt: CommandCtxt<'_>, input: ImageUrl) -> anyhow::Result<()> {
    let result = identify_song_notsoidentify(&ctxt.assyst().reqwest_client, input.0)
        .await
        .context("Failed to identify song")?;

    if !result.is_empty() {
        let formatted = format!(
            "**Title:** {}\n**Artist(s):** {}\n**YouTube Link:** <{}>",
            result[0].title.clone(),
            result[0]
                .artists
                .iter()
                .map(|x| x.name.clone())
                .collect::<Vec<_>>()
                .join(", "),
            match &result[0].platforms.youtube {
                Some(x) => x.url.clone(),
                None => "Unknown".to_owned(),
            }
        );
        ctxt.reply(formatted).await?;
    } else {
        ctxt.reply("No results found").await?;
    }

    Ok(())
}
