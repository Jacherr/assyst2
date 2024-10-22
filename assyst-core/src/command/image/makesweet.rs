use std::time::Duration;

use assyst_proc_macro::command;

use crate::command::arguments::{Image, RestNoFlags};
use crate::command::{Availability, Category, CommandCtxt};

#[command(
    name = "backtattoo",
    description = "put an image on a person's back",
    cooldown = Duration::from_secs(3),
    access = Availability::Public,
    category = Category::Makesweet,
    usage = "[image]",
    examples = ["https://link.to.my/image.png"],
    send_processing = true
)]
pub async fn back_tattoo(ctxt: CommandCtxt<'_>, source: Image) -> anyhow::Result<()> {
    let result = ctxt
        .flux_handler()
        .back_tattoo(source.0, ctxt.data.author.id.get(), ctxt.data.guild_id.map(twilight_model::id::Id::get))
        .await?;

    ctxt.reply(result).await?;

    Ok(())
}

#[command(
    description = "put an image on a billboard",
    cooldown = Duration::from_secs(3),
    access = Availability::Public,
    category = Category::Makesweet,
    usage = "[image]",
    examples = ["https://link.to.my/image.png"],
    send_processing = true
)]
pub async fn billboard(ctxt: CommandCtxt<'_>, source: Image) -> anyhow::Result<()> {
    let result = ctxt
        .flux_handler()
        .billboard(source.0, ctxt.data.author.id.get(), ctxt.data.guild_id.map(twilight_model::id::Id::get))
        .await?;

    ctxt.reply(result).await?;

    Ok(())
}

#[command(
    description = "put an image in a book",
    cooldown = Duration::from_secs(3),
    access = Availability::Public,
    category = Category::Makesweet,
    usage = "[image]",
    examples = ["https://link.to.my/image.png"],
    send_processing = true
)]
pub async fn book(ctxt: CommandCtxt<'_>, source: Image) -> anyhow::Result<()> {
    let result = ctxt
        .flux_handler()
        .book(source.0, ctxt.data.author.id.get(), ctxt.data.guild_id.map(twilight_model::id::Id::get))
        .await?;

    ctxt.reply(result).await?;

    Ok(())
}

#[command(
    description = "put an image on a circuit board",
    cooldown = Duration::from_secs(3),
    access = Availability::Public,
    category = Category::Makesweet,
    usage = "[image]",
    examples = ["https://link.to.my/image.png"],
    send_processing = true
)]
pub async fn circuitboard(ctxt: CommandCtxt<'_>, source: Image) -> anyhow::Result<()> {
    let result = ctxt
        .flux_handler()
        .circuitboard(source.0, ctxt.data.author.id.get(), ctxt.data.guild_id.map(twilight_model::id::Id::get))
        .await?;

    ctxt.reply(result).await?;

    Ok(())
}

#[command(
    description = "put an image on a flag",
    cooldown = Duration::from_secs(3),
    access = Availability::Public,
    category = Category::Makesweet,
    usage = "[image]",
    examples = ["https://link.to.my/image.png"],
    send_processing = true
)]
pub async fn flag(ctxt: CommandCtxt<'_>, source: Image) -> anyhow::Result<()> {
    let result = ctxt
        .flux_handler()
        .flag(source.0, ctxt.data.author.id.get(), ctxt.data.guild_id.map(twilight_model::id::Id::get))
        .await?;

    ctxt.reply(result).await?;

    Ok(())
}

#[command(
    description = "put an image on a different flag",
    cooldown = Duration::from_secs(3),
    access = Availability::Public,
    category = Category::Makesweet,
    usage = "[image]",
    examples = ["https://link.to.my/image.png"],
    send_processing = true
)]
pub async fn flag2(ctxt: CommandCtxt<'_>, source: Image) -> anyhow::Result<()> {
    let result = ctxt
        .flux_handler()
        .flag2(source.0, ctxt.data.author.id.get(), ctxt.data.guild_id.map(twilight_model::id::Id::get))
        .await?;

    ctxt.reply(result).await?;

    Ok(())
}

#[command(
    description = "put an image in a fortune cookie",
    cooldown = Duration::from_secs(3),
    access = Availability::Public,
    category = Category::Makesweet,
    usage = "[image]",
    examples = ["https://link.to.my/image.png"],
    send_processing = true
)]
pub async fn fortunecookie(ctxt: CommandCtxt<'_>, source: Image) -> anyhow::Result<()> {
    let result = ctxt
        .flux_handler()
        .fortune_cookie(source.0, ctxt.data.author.id.get(), ctxt.data.guild_id.map(twilight_model::id::Id::get))
        .await?;

    ctxt.reply(result).await?;

    Ok(())
}

#[command(
    description = "put an image in a heart locket with some text",
    cooldown = Duration::from_secs(3),
    access = Availability::Public,
    category = Category::Makesweet,
    usage = "[image] [text]",
    examples = ["https://link.to.my/image.png hello"],
    send_processing = true
)]
pub async fn heartlocket(ctxt: CommandCtxt<'_>, source: Image, text: RestNoFlags) -> anyhow::Result<()> {
    let result = ctxt
        .flux_handler()
        .heart_locket(
            source.0,
            text.0,
            ctxt.data.author.id.get(),
            ctxt.data.guild_id.map(twilight_model::id::Id::get),
        )
        .await?;

    ctxt.reply(result).await?;

    Ok(())
}

#[command(
    description = "put an image on a rubik's cube",
    aliases = ["rubix"],
    cooldown = Duration::from_secs(3),
    access = Availability::Public,
    category = Category::Makesweet,
    usage = "[image]",
    examples = ["https://link.to.my/image.png"],
    send_processing = true
)]
pub async fn rubiks(ctxt: CommandCtxt<'_>, source: Image) -> anyhow::Result<()> {
    let result = ctxt
        .flux_handler()
        .rubiks(source.0, ctxt.data.author.id.get(), ctxt.data.guild_id.map(twilight_model::id::Id::get))
        .await?;

    ctxt.reply(result).await?;

    Ok(())
}

#[command(
    description = "put an image on a toaster",
    cooldown = Duration::from_secs(3),
    access = Availability::Public,
    category = Category::Makesweet,
    usage = "[image]",
    examples = ["https://link.to.my/image.png"],
    send_processing = true
)]
pub async fn toaster(ctxt: CommandCtxt<'_>, source: Image) -> anyhow::Result<()> {
    let result = ctxt
        .flux_handler()
        .toaster(source.0, ctxt.data.author.id.get(), ctxt.data.guild_id.map(twilight_model::id::Id::get))
        .await?;

    ctxt.reply(result).await?;

    Ok(())
}

#[command(
    description = "display an image as a valentine's gift",
    cooldown = Duration::from_secs(3),
    access = Availability::Public,
    category = Category::Makesweet,
    usage = "[image]",
    examples = ["https://link.to.my/image.png"],
    send_processing = true
)]
pub async fn valentine(ctxt: CommandCtxt<'_>, source: Image) -> anyhow::Result<()> {
    let result = ctxt
        .flux_handler()
        .valentine(source.0, ctxt.data.author.id.get(), ctxt.data.guild_id.map(twilight_model::id::Id::get))
        .await?;

    ctxt.reply(result).await?;

    Ok(())
}
