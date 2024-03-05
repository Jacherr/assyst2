use std::time::{Duration, Instant};

use crate::command::Availability;

use super::arguments::{Image, ImageUrl, Rest, Time, Word};
use super::{Category, CommandCtxt};

use assyst_common::ansi::Ansi;
use assyst_common::markdown::Markdown;
use assyst_common::util::format_duration;
use assyst_common::util::process::exec_sync;
use assyst_common::util::table::key_value;
use assyst_proc_macro::command;

pub mod help;
pub mod stats;
pub mod tag;

#[command(
    name = "remind",
    aliases = ["reminder"],
    description = "get reminders or set a reminder, time format is xdyhzm (check examples)",
    access = Availability::Public,
    cooldown = Duration::from_secs(2),
    category = Category::Misc,
    usage = "[time] <message>",
    examples = ["2h do the laundry", "3d30m hand assignment in", "30m"],
)]
pub async fn remind(_ctxt: CommandCtxt<'_>, _when: Time, _text: Rest) -> anyhow::Result<()> {
    Ok(())
}

#[command(
    description = "enlarges an image", 
    aliases = ["e"], 
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
        ("Full Processing Time".fg_cyan(), processing_time.to_string()),
        ("Metadata and Args Parsing".fg_cyan(), metadata_time.to_string()),
        ("Preprocessing Time".fg_cyan(), preprocess_time.to_string()),
        ("Prefix Determinism Time".fg_cyan(), prefix_time.to_string()),
        ("Full Parsing Time".fg_cyan(), parse_time.to_string()),
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
