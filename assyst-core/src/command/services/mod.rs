use std::time::Duration;

use assyst_common::util::format_duration;
use assyst_proc_macro::command;

use super::arguments::{Rest, Word};
use super::CommandCtxt;

use crate::command::{Availability, Category};
use crate::rest::cooltext::burn_text;
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
    usage = "[url] <audio|[quality]>",
    examples = ["https://www.youtube.com/watch?v=dQw4w9WgXcQ", "https://www.youtube.com/watch?v=dQw4w9WgXcQ audio", "https://www.youtube.com/watch?v=dQw4w9WgXcQ 480"],
    send_processing = true
)]
pub async fn download(ctxt: CommandCtxt<'_>, url: Word, opts_str: Option<Rest>) -> anyhow::Result<()> {
    let mut opts = WebDownloadOpts::default();
    if let Some(opts_str) = opts_str {
        for opt in opts_str.0.split_whitespace() {
            let trim = opt.trim();
            if trim == "audio" {
                opts.audio_only = Some(true);
            } else if let Ok(i) = trim.parse::<u16>()
                && opts.quality.is_none()
            {
                opts.quality = Some(i.to_string());
            }
        }
    }

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
