use std::time::Duration;

use anyhow::Context;
use assyst_proc_macro::command;

use super::arguments::{Image, Rest, Word};
use crate::command::{Availability, Category, CommandCtxt};

#[command(
    description = "ah shit here we go again",
    cooldown = Duration::from_secs(2),
    access = Availability::Public,
    category = Category::Wsi,
    usage = "[image]",
    examples = ["https://link.to.my/image.png"],
    send_processing = true
)]
pub async fn ahshit(ctxt: CommandCtxt<'_>, source: Image) -> anyhow::Result<()> {
    let result = ctxt.wsi_handler().ahshit(source.0, ctxt.data.author.id.get()).await?;

    ctxt.reply(result).await?;

    Ok(())
}

#[command(
    description = "april fools!!!!",
    cooldown = Duration::from_secs(2),
    access = Availability::Public,
    category = Category::Wsi,
    usage = "[image]",
    examples = ["https://link.to.my/image.png"],
    send_processing = true
)]
pub async fn aprilfools(ctxt: CommandCtxt<'_>, source: Image) -> anyhow::Result<()> {
    let result = ctxt
        .wsi_handler()
        .aprilfools(source.0, ctxt.data.author.id.get())
        .await?;

    ctxt.reply(result).await?;

    Ok(())
}

#[command(
    description = "bloom an image",
    cooldown = Duration::from_secs(2),
    access = Availability::Public,
    category = Category::Wsi,
    usage = "[image] <radius>",
    examples = ["https://link.to.my/image.png"],
    send_processing = true
)]
pub async fn bloom(ctxt: CommandCtxt<'_>, source: Image, radius: Option<u64>) -> anyhow::Result<()> {
    let result = ctxt
        .wsi_handler()
        .bloom(source.0, radius.unwrap_or(5) as usize, ctxt.data.author.id.get())
        .await?;

    ctxt.reply(result).await?;

    Ok(())
}

#[command(
    description = "blur an image",
    cooldown = Duration::from_secs(2),
    access = Availability::Public,
    category = Category::Wsi,
    usage = "[image] <radius>",
    examples = ["https://link.to.my/image.png"],
    send_processing = true
)]
pub async fn blur(ctxt: CommandCtxt<'_>, source: Image, strength: Option<f32>) -> anyhow::Result<()> {
    let result = ctxt
        .wsi_handler()
        .blur(source.0, strength.unwrap_or(1.0), ctxt.data.author.id.get())
        .await?;

    ctxt.reply(result).await?;

    Ok(())
}

#[command(
    description = "add a caption to an image",
    cooldown = Duration::from_secs(2),
    access = Availability::Public,
    category = Category::Wsi,
    usage = "[image] [caption]",
    examples = ["https://link.to.my/image.png hello there"],
    send_processing = true
)]
pub async fn caption(ctxt: CommandCtxt<'_>, source: Image, text: Rest) -> anyhow::Result<()> {
    let result = ctxt
        .wsi_handler()
        .caption(source.0, text.0, ctxt.data.author.id.get())
        .await?;

    ctxt.reply(result).await?;

    Ok(())
}

#[command(
    description = "resize an image based on scale or WxH (default 2x)",
    cooldown = Duration::from_secs(2),
    access = Availability::Public,
    category = Category::Wsi,
    usage = "[image] <scale>",
    examples = ["https://link.to.my/image.png", "https://link.to.my/image.png 128x128", "https://link.to.my/image.png 2"],
    send_processing = true
)]
pub async fn resize(ctxt: CommandCtxt<'_>, source: Image, size: Option<Word>) -> anyhow::Result<()> {
    let result = if let Some(ref i_size) = size
        && i_size.0.contains('x')
        && let Some((width, height)) = i_size.0.split_once('x')
    {
        let width = width.parse::<u32>().context("Invalid width.")?;
        let height = height.parse::<u32>().context("Invalid height.")?;

        ctxt.wsi_handler()
            .resize_absolute(source.0, width, height, ctxt.data.author.id.get())
            .await?
    } else if let Some(i_size) = size {
        let scale = i_size.0.parse::<f32>().context("Invalid scale.")?;

        ctxt.wsi_handler()
            .resize_scale(source.0, scale, ctxt.data.author.id.get())
            .await?
    } else {
        ctxt.wsi_handler()
            .resize_scale(source.0, 2.0, ctxt.data.author.id.get())
            .await?
    };

    ctxt.reply(result).await?;

    Ok(())
}
