use std::time::Duration;

use assyst_proc_macro::command;

use crate::command::arguments::Image;
use crate::command::{Availability, Category, CommandCtxt};

#[command(
    description = "give an image drip",
    cooldown = Duration::from_secs(3),
    access = Availability::Public,
    category = Category::Audio,
    usage = "[image]",
    examples = ["https://link.to.my/image.png"],
    send_processing = true
)]
pub async fn drip(ctxt: CommandCtxt<'_>, source: Image) -> anyhow::Result<()> {
    let result = ctxt
        .flux_handler()
        .drip(source.0, ctxt.data.author.id.get(), ctxt.data.guild_id.map(twilight_model::id::Id::get))
        .await?;

    ctxt.reply(result).await?;

    Ok(())
}

#[command(
    description = "femurbreaker over image",
    cooldown = Duration::from_secs(3),
    access = Availability::Public,
    category = Category::Audio,
    usage = "[image]",
    examples = ["https://link.to.my/image.png"],
    send_processing = true
)]
pub async fn femurbreaker(ctxt: CommandCtxt<'_>, source: Image) -> anyhow::Result<()> {
    let result = ctxt
        .flux_handler()
        .femurbreaker(source.0, ctxt.data.author.id.get(), ctxt.data.guild_id.map(twilight_model::id::Id::get))
        .await?;

    ctxt.reply(result).await?;

    Ok(())
}

#[command(
    description = "⚠️ alert ⚠️",
    cooldown = Duration::from_secs(3),
    access = Availability::Public,
    category = Category::Audio,
    usage = "[image]",
    examples = ["https://link.to.my/image.png"],
    send_processing = true
)]
pub async fn siren(ctxt: CommandCtxt<'_>, source: Image) -> anyhow::Result<()> {
    let result = ctxt
        .flux_handler()
        .siren(source.0, ctxt.data.author.id.get(), ctxt.data.guild_id.map(twilight_model::id::Id::get))
        .await?;

    ctxt.reply(result).await?;

    Ok(())
}

#[command(
    description = "give an image some minecraft nostalgia",
    cooldown = Duration::from_secs(3),
    access = Availability::Public,
    category = Category::Audio,
    usage = "[image]",
    examples = ["https://link.to.my/image.png"],
    send_processing = true
)]
pub async fn sweden(ctxt: CommandCtxt<'_>, source: Image) -> anyhow::Result<()> {
    let result = ctxt
        .flux_handler()
        .sweden(source.0, ctxt.data.author.id.get(), ctxt.data.guild_id.map(twilight_model::id::Id::get))
        .await?;

    ctxt.reply(result).await?;

    Ok(())
}

#[command(
    description = "give your image a grassy theme tune",
    cooldown = Duration::from_secs(3),
    access = Availability::Public,
    category = Category::Audio,
    usage = "[image]",
    examples = ["https://link.to.my/image.png"],
    send_processing = true
)]
pub async fn terraria(ctxt: CommandCtxt<'_>, source: Image) -> anyhow::Result<()> {
    let result = ctxt
        .flux_handler()
        .terraria(source.0, ctxt.data.author.id.get(), ctxt.data.guild_id.map(twilight_model::id::Id::get))
        .await?;

    ctxt.reply(result).await?;

    Ok(())
}
