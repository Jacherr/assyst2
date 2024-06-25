use std::time::Duration;

use anyhow::{bail, Context};
use assyst_common::markdown::Markdown;
use assyst_common::util::discord::format_discord_timestamp;
use assyst_common::util::{format_time, table};
use assyst_database::model::reminder::Reminder;
use assyst_proc_macro::command;

use crate::command::arguments::{Rest, Time};
use crate::command::{Availability, Category, CommandCtxt};
use crate::define_commandgroup;

#[command(
    aliases = ["reminder"],
    description = "set a new reminder - time format is xdyhzm (check examples)",
    access = Availability::Public,
    cooldown = Duration::from_secs(2),
    category = Category::Misc,
    usage = "[time] <message>",
    examples = ["2h do the laundry", "3d30m hand assignment in", "30m"],
)]
pub async fn default(ctxt: CommandCtxt<'_>, when: Time, text: Option<Rest>) -> anyhow::Result<()> {
    let text = text.map(|x| x.0).unwrap_or("...".to_owned());

    if text.len() > 250 {
        bail!("Reminder message cannot exceed 250 characters.");
    }

    let reminder = Reminder {
        id: 0, // unused
        user_id: ctxt.data.author.id.get() as i64,
        timestamp: when.millis as i64,
        guild_id: ctxt.data.guild_id.map(|x| x.get()).unwrap_or(0) as i64,
        channel_id: ctxt.data.channel_id.get() as i64,
        message_id: ctxt.data.message.map(|x| x.id.get()).unwrap_or(0) as i64,
        message: text,
    };

    reminder
        .insert(&ctxt.assyst().database_handler)
        .await
        .context("Failed to insert reminder to database")?;

    ctxt.reply(format!(
        "Reminder successfully set for {} from now.",
        format_time(when.millis)
    ))
    .await?;

    Ok(())
}

#[command(
    description = "list all of your reminders",
    access = Availability::Public,
    cooldown = Duration::from_secs(2),
    category = Category::Misc,
    usage = "",
    examples = [""],
)]
pub async fn list(ctxt: CommandCtxt<'_>) -> anyhow::Result<()> {
    let reminders = Reminder::fetch_user_reminders(&ctxt.assyst().database_handler, ctxt.data.author.id.get(), 10)
        .await
        .context("Failed to fetch reminders")?;

    if reminders.is_empty() {
        ctxt.reply("You don't have any set reminders.").await?;
        return Ok(());
    }

    let formatted = reminders
        .iter()
        .map(|reminder| {
            format!(
                "[#{}] {}: `{}`\n",
                reminder.id,
                format_discord_timestamp(reminder.timestamp as u64),
                reminder.message
            )
        })
        .collect::<String>();

    ctxt.reply(format!(":calendar: **Upcoming Reminders:**\n\n{formatted}"))
        .await?;

    Ok(())
}

define_commandgroup! {
    name: remind,
    access: Availability::Public,
    category: Category::Misc,
    aliases: ["t"],
    description: "assyst reminders - get and set reminders",
    examples: ["2h do the laundry", "3d30m hand assignment in", "30m"],
    usage: "[time] <message>",
    commands: [
        "list" => list
    ],
    default_interaction_subcommand: "create",
    default: default
}
