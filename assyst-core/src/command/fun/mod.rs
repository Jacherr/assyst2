use std::time::Duration;

use anyhow::{bail, Context};
use assyst_proc_macro::command;

use super::arguments::ImageUrl;
use super::CommandCtxt;
use crate::command::{Availability, Category};
use crate::rest::audio_identification::identify_song_notsoidentify;
use crate::rest::identify::identify_image;

pub mod colour;
pub mod translation;

#[command(
    description = "Find a song in a video",
    cooldown = Duration::from_secs(2),
    access = Availability::Public,
    category = Category::Fun,
    usage = "[video]",
    examples = ["https://link.to.my/video.mp4"],
    send_processing = true,
    context_menu_message_command = "Find Song"
)]
pub async fn findsong(ctxt: CommandCtxt<'_>, audio: ImageUrl) -> anyhow::Result<()> {
    const VALID_FILES: &[&str] = &["mp3", "mp4", "webm", "ogg", "wav"];

    if VALID_FILES.iter().all(|x| !audio.0.ends_with(x)) {
        bail!("Finding audio is only supported on audio and video files.");
    }

    let result = identify_song_notsoidentify(&ctxt.assyst().reqwest_client, audio.0)
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

#[command(
    description = "AI identify an image",
    cooldown = Duration::from_secs(2),
    access = Availability::Public,
    category = Category::Fun,
    usage = "[image]",
    examples = ["https://link.to.my/image.png"],
    send_processing = true,
    context_menu_command = "Identify Image"
)]
pub async fn identify(ctxt: CommandCtxt<'_>, input: ImageUrl) -> anyhow::Result<()> {
    let result = identify_image(&ctxt.assyst().reqwest_client, &input.0)
        .await
        .context("Failed to identify image")?;

    if let Some(d) = result.description {
        let formatted = d
            .captions
            .iter()
            .map(|x| format!("I think it's {} ({:.1}% confidence)", x.text, (x.confidence * 100.0)))
            .collect::<Vec<_>>()
            .join("\n");

        ctxt.reply(formatted).await?;
    } else {
        ctxt.reply("I really can't describe the picture :flushed:").await?;
    }

    Ok(())
}
