use std::time::Duration;

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
pub fn remind(ctxt: CommandCtxt<'_>, when: Time, text: Rest) -> anyhow::Result<()> {
    Ok(())
}

#[command(description = "", cooldown = Duration::ZERO, access = Availability::Public)]
pub fn e(ctxt: CommandCtxt<'_>, source: Image) -> anyhow::Result<()> {
    Ok(())
}
