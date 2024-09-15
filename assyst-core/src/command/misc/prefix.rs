use std::time::Duration;

use anyhow::{bail, ensure, Context};
use assyst_database::model::prefix::Prefix;
use assyst_proc_macro::command;
use assyst_string_fmt::Markdown;

use crate::command::arguments::Word;
use crate::command::{Availability, Category, CommandCtxt};
use crate::define_commandgroup;

#[command(
    description = "get server prefix",
    access = Availability::Public,
    cooldown = Duration::from_secs(2),
    category = Category::Misc,
    examples = [""],
)]
pub async fn default(ctxt: CommandCtxt<'_>) -> anyhow::Result<()> {
    let Some(guild_id) = ctxt.data.guild_id else {
        bail!("prefix getting and setting can only be used in guilds")
    };

    let prefix = Prefix::get(&ctxt.assyst().database_handler, guild_id.get())
        .await
        .context("Failed to get guild prefix")?
        .context("This guild has no set prefix?")?;

    ctxt.reply(format!("This server's prefix is: {}", prefix.prefix.codestring()))
        .await?;

    Ok(())
}

#[command(
    description = "set server prefix",
    access = Availability::ServerManagers,
    cooldown = Duration::from_secs(2),
    category = Category::Misc,
    examples = ["-", "%"],
)]
pub async fn set(ctxt: CommandCtxt<'_>, new: Word) -> anyhow::Result<()> {
    let Some(guild_id) = ctxt.data.guild_id else {
        bail!("Prefix getting and setting can only be used in guilds.")
    };

    ensure!(new.0.len() < 14, "Prefixes cannot be longer than 14 characters.");

    let new = Prefix { prefix: new.0 };
    new.set(&ctxt.assyst().database_handler, guild_id.get())
        .await
        .context("Failed to set new prefix")?;

    ctxt.reply(format!("This server's prefix is now: {}", new.prefix.codestring()))
        .await?;

    Ok(())
}

define_commandgroup! {
    name: prefix,
    access: Availability::Public,
    category: Category::Misc,
    description: "get or set server prefix",
    usage: "<set> <new prefix>",
    commands: [
        "set" => set
    ],
    default_interaction_subcommand: "get",
    default: default
}
