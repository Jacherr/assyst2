use std::time::Duration;

use anyhow::bail;
use assyst_common::util::format_duration;
use assyst_proc_macro::command;
use rand::seq::SliceRandom;
use rand::thread_rng;

use super::arguments::{Rest, Word};
use super::flags::DownloadFlags;
use super::CommandCtxt;

use crate::command::{Availability, Category};
use crate::rest::cooltext::{burn_text, STYLES};
use crate::rest::r34::get_random_r34;
use crate::rest::web_media_download::{download_web_media, WebDownloadOpts};

#[command(
    aliases = ["firetext"],
    description = "make some burning text",
    access = Availability::Public,
    cooldown = Duration::from_secs(2),
    category = Category::Services,
    usage = "[text]",
    examples = ["yep im burning"],
    send_processing = true
)]
pub async fn burntext(ctxt: CommandCtxt<'_>, text: Rest) -> anyhow::Result<()> {
    let result = burn_text(&text.0).await?;

    ctxt.reply(result).await?;

    Ok(())
}

#[command(
    description = "make some cool text",
    access = Availability::Public,
    cooldown = Duration::from_secs(2),
    category = Category::Services,
    examples = ["burning hello", "saint fancy", "random im random"],
    send_processing = true
)]
pub async fn cooltext(ctxt: CommandCtxt<'_>, style: Word, text: Rest) -> anyhow::Result<()> {
    let style = if &style.0 == "random" {
        let mut s = STYLES.to_vec();
        s.shuffle(&mut thread_rng());
        s[0].0
    } else {
        &style.0
    };

    let result = crate::rest::cooltext::cooltext(style, text.0.as_str()).await;
    if let Ok(r) = result {
        ctxt.reply((r, &format!("**Style:** `{style}`")[..])).await?;
    } else {
        bail!(
            "unknown style {}, available styles are: {}",
            style,
            STYLES.iter().map(|(v, _)| *v).collect::<Vec<_>>().join(", ")
        )
    }

    Ok(())
}

#[command(
    name = "rule34",
    aliases = ["r34"],
    description = "get random image from r34",
    access = Availability::Public,
    cooldown = Duration::from_secs(2),
    category = Category::Services,
    usage = "<tags separated by spaces>",
    examples = ["", "assyst"],
    age_restricted = true
)]
pub async fn r34(ctxt: CommandCtxt<'_>, tags: Rest) -> anyhow::Result<()> {
    let result = get_random_r34(ctxt.assyst().clone(), &tags.0).await?;
    let reply = format!("{} (Score: **{}**)", result.file_url, result.score);

    ctxt.reply(reply).await?;

    Ok(())
}

#[command(
    name = "download",
    aliases = ["dl"],
    description = "download media from a website",
    access = Availability::Public,
    cooldown = Duration::from_secs(2),
    category = Category::Services,
    usage = "[url] <flags>",
    examples = ["https://www.youtube.com/watch?v=dQw4w9WgXcQ", "https://www.youtube.com/watch?v=dQw4w9WgXcQ --audio", "https://www.youtube.com/watch?v=dQw4w9WgXcQ --quality 480"],
    send_processing = true,
    flag_descriptions = [
        ("audio", "Get content as MP3"),
        ("quality [quality:144|240|360|480|720|1080|max]", "Set resolution of output"),
    ]
)]
pub async fn download(ctxt: CommandCtxt<'_>, url: Word, options: DownloadFlags) -> anyhow::Result<()> {
    let opts = WebDownloadOpts::from_download_flags(options);

    let result = download_web_media(ctxt.assyst().clone(), &url.0, opts).await?;

    ctxt.reply((
        result,
        &format!(
            "Took {}",
            format_duration(&ctxt.data.execution_timings.processing_time_start.elapsed())
        )[..],
    ))
    .await?;

    Ok(())
}
