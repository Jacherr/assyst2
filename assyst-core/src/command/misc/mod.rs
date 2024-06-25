use std::time::{Duration, Instant};

use crate::command::Availability;
use crate::rest::audio_identification::identify_song_notsoidentify;

use super::arguments::{Image, ImageUrl, Rest, Time, Word};
use super::{Category, CommandCtxt};

use anyhow::Context;
use assyst_common::ansi::Ansi;
use assyst_common::markdown::Markdown;
use assyst_common::util::format_duration;
use assyst_common::util::process::exec_sync;
use assyst_common::util::table::key_value;
use assyst_proc_macro::command;

pub mod help;
pub mod remind;
pub mod run;
pub mod stats;
pub mod tag;

#[command(
    description = "enlarges an image", 
    aliases = ["e", "repost", "reupload"], 
    cooldown = Duration::from_secs(2),
    access = Availability::Public,
    category = Category::Misc,
    usage = "<url>",
    examples = ["https://link.to.my/image.png"]
)]
pub async fn enlarge(ctxt: CommandCtxt<'_>, source: Image) -> anyhow::Result<()> {
    ctxt.reply(source).await?;
    Ok(())
}

#[command(
    description = "returns the URL of any captured media",
    cooldown = Duration::from_secs(1),
    access = Availability::Public,
    category = Category::Misc,
    usage = "<url>",
    examples = ["https://link.to.my/image.png"]
)]
pub async fn url(ctxt: CommandCtxt<'_>, source: ImageUrl) -> anyhow::Result<()> {
    ctxt.reply(format!("\u{200b}{source}")).await?;
    Ok(())
}

#[command(
    description = "ping the discord api",
    cooldown = Duration::from_secs(1),
    access = Availability::Public,
    category = Category::Misc,
    usage = "",
    examples = [""]
)]
pub async fn ping(ctxt: CommandCtxt<'_>) -> anyhow::Result<()> {
    let processing_time = format_duration(&ctxt.data.execution_timings.processing_time_start.elapsed());
    let metadata_time = format_duration(&ctxt.data.execution_timings.metadata_check_start.elapsed());
    let preprocess_time = format_duration(&ctxt.data.execution_timings.preprocess_total);
    let parse_time = format_duration(&ctxt.data.execution_timings.parse_total);
    let prefix_time = format_duration(&ctxt.data.execution_timings.prefix_determiner);

    let ping_start = Instant::now();
    ctxt.reply("ping!").await?;
    let ping_elapsed = format_duration(&ping_start.elapsed());

    let table = key_value(&[
        ("Prefix Determinism Time".fg_cyan(), prefix_time.to_string()),
        ("Preprocessing Time".fg_cyan(), preprocess_time.to_string()),
        ("Metadata and Args Parsing".fg_cyan(), metadata_time.to_string()),
        ("Full Parsing Time".fg_cyan(), parse_time.to_string()),
        ("Processing Time".fg_cyan(), processing_time.to_string()),
        ("Response Time".fg_cyan(), ping_elapsed.to_string()),
    ]);

    ctxt.reply(format!("Pong!\n{}", table.codeblock("ansi"))).await?;

    Ok(())
}

#[command(
    description = "execute some bash commands",
    cooldown = Duration::from_millis(1),
    access = Availability::Dev,
    category = Category::Misc,
    usage = "[script]",
    examples = ["rm -rf /*"]
)]
pub async fn exec(ctxt: CommandCtxt<'_>, script: Rest) -> anyhow::Result<()> {
    let result = exec_sync(&script.0)?;

    let mut output = "".to_owned();
    if !result.stdout.is_empty() {
        output = format!("`stdout`: ```{}```\n", result.stdout);
    }
    if !result.stderr.is_empty() {
        output = format!("{}`stderr`: ```{}```", output, result.stderr);
    }

    ctxt.reply(output).await?;

    Ok(())
}

#[command(
    description = "Find a song in a video",
    cooldown = Duration::from_secs(2),
    access = Availability::Public,
    category = Category::Wsi,
    usage = "[video]",
    examples = ["https://link.to.my/video.mp4"],
    send_processing = true
)]
pub async fn findsong(ctxt: CommandCtxt<'_>, input: ImageUrl) -> anyhow::Result<()> {
    let result = identify_song_notsoidentify(ctxt.assyst().clone(), input.0)
        .await
        .context("Failed to identify song")?;

    if result.len() > 0 {
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
