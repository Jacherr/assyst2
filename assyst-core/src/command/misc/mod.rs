use std::time::{Duration, Instant};

use crate::command::Availability;

use super::arguments::{Image, Rest, Time};
use super::CommandCtxt;

use assyst_proc_macro::command;

#[command(
    name = "remind",
    aliases = ["reminder"],
    description = "get reminders or set a reminder, time format is xdyhzm (check examples)",
    access = Availability::Public,
    cooldown = Duration::from_secs(2)
)]
pub async fn remind(_ctxt: CommandCtxt<'_>, _when: Time, _text: Rest) -> anyhow::Result<()> {
    Ok(())
}

#[command(description = "", cooldown = Duration::ZERO, access = Availability::Public)]
pub async fn e(ctxt: CommandCtxt<'_>, source: Image) -> anyhow::Result<()> {
    ctxt.reply(source).await?;
    Ok(())
}

#[command(description = "", cooldown = Duration::ZERO, access = Availability::Public)]
pub async fn ping(ctxt: CommandCtxt<'_>) -> anyhow::Result<()> {
    let processing_time = ctxt.data.processing_time_start.elapsed();
    let ping_start = Instant::now();
    ctxt.reply("ping!").await?;
    let ping_elapsed = ping_start.elapsed();
    ctxt.reply(format!(
        "pong!\nprocessing time: {processing_time:?}\nresponse time: {ping_elapsed:?}"
    ))
    .await?;

    Ok(())
}
