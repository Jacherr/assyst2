use std::time::Duration;

use anyhow::{bail, ensure, Context};
use assyst_common::util::discord::ensure_same_guild;
use assyst_database::model::badtranslator_channel::BadTranslatorChannel;
use assyst_proc_macro::command;

use crate::command::arguments::{Channel, Word};
use crate::command::{Availability, Category, CommandCtxt};
use crate::define_commandgroup;
use crate::rest::bad_translation::validate_language;

#[command(
    description = "register a new badtranslator channel",
    aliases = ["create"],
    cooldown = Duration::from_secs(10),
    access = Availability::ServerManagers,
    category = Category::Misc,
    usage = "[channel] [language]",
    examples = ["#bt en"],
    guild_only = true
)]
pub async fn add(ctxt: CommandCtxt<'_>, channel: Channel, target_language: Word) -> anyhow::Result<()> {
    let Some(guild_id) = ctxt.data.guild_id else {
        bail!("BadTranslator channels can ony be created inside of servers.");
    };

    ensure_same_guild(&ctxt.assyst().http_client, channel.0.id.get(), guild_id.get()).await?;
    ensure!(
        validate_language(&ctxt.assyst().reqwest_client, &target_language.0).await?,
        "This language does not exist or cannot be used as a target language. Run `{}btchannel languages` for a list of languages",
        ctxt.data.calling_prefix
    );

    ctxt.assyst()
        .http_client
        .create_webhook(channel.0.id, "Bad Translator")
        .await
        .context("Failed to create BadTranslator webhook")?;

    let new = BadTranslatorChannel {
        id: channel.0.id.get() as i64,
        target_language: target_language.0,
    };

    ensure!(
        new.set(&ctxt.assyst().database_handler)
            .await
            .context("Failed to register BadTranslator channel")?,
        "This channel is already registered as a BadTranslator channel."
    );

    ctxt.assyst()
        .bad_translator
        .add_channel(channel.0.id.get(), &new.target_language)
        .await;

    ctxt.reply(format!(
        "The channel <#{}> has been registered as a BadTranslator channel.",
        channel.0.id.get()
    ))
    .await?;

    Ok(())
}

#[command(
    description = "delete an existing badtranslator channel",
    aliases = ["remove"],
    cooldown = Duration::from_secs(10),
    access = Availability::ServerManagers,
    category = Category::Misc,
    usage = "[channel]",
    examples = ["#bt"],
    guild_only = true
)]
pub async fn remove(ctxt: CommandCtxt<'_>, channel: Channel) -> anyhow::Result<()> {
    let Some(guild_id) = ctxt.data.guild_id else {
        bail!("BadTranslator channels can ony be deleted inside of servers.");
    };

    ensure_same_guild(&ctxt.assyst().http_client, channel.0.id.get(), guild_id.get()).await?;

    let webhooks = ctxt
        .assyst()
        .http_client
        .channel_webhooks(channel.0.id)
        .await
        .context("Failed to get webhooks")?
        .model()
        .await
        .context("Failed to get webhooks")?;

    for webhook in webhooks {
        if webhook.name == Some("Bad Translator".to_owned()) {
            ctxt.assyst()
                .http_client
                .delete_webhook(webhook.id)
                .await
                .context("Failed to delete BadTranslator webhook")?;
        }
    }

    ensure!(
        BadTranslatorChannel::delete(&ctxt.assyst().database_handler, channel.0.id.get() as i64)
            .await
            .context("Failed to delete BadTranslator channel")?,
        "Failed to delete this BadTranslator channel - does it exist?"
    );

    ctxt.assyst().bad_translator.remove_bt_channel(channel.0.id.get()).await;

    ctxt.reply(format!(
        "The channel <#{}> has been unregistered as a BadTranslator channel.",
        channel.0.id.get()
    ))
    .await?;

    Ok(())
}

define_commandgroup! {
    name: btchannel,
    access: Availability::ServerManagers,
    category: Category::Misc,
    description: "manage badtranslator channels",
    usage: "[subcommand] <arguments...>",
    guild_only: true,
    commands: [
        "add" => add,
        "remove" => remove
    ]
}
