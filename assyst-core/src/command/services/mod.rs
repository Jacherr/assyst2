use std::time::Duration;

use assyst_proc_macro::command;

use super::arguments::Rest;
use super::CommandCtxt;
use crate::command::{Availability, Category};
use crate::rest::cooltext::burn_text;
use crate::rest::r34::get_random_r34;

pub mod cooltext;
pub mod download;

#[command(
    aliases = ["firetext"],
    description = "make some burning text",
    access = Availability::Public,
    cooldown = Duration::from_secs(2),
    category = Category::Services,
    usage = "[text]",
    examples = ["yep im burning"],
    send_processing = true,
    context_menu_message_command = "Burn Text"
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
pub async fn r34(ctxt: CommandCtxt<'_>, tags: Option<Rest>) -> anyhow::Result<()> {
    let result = get_random_r34(&ctxt.assyst().reqwest_client, &tags.unwrap_or(Rest(String::new())).0).await?;
    let reply = format!("{} (Score: **{}**)", result.file_url, result.score);

    ctxt.reply(reply).await?;

    Ok(())
}
