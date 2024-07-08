use std::time::Duration;

use anyhow::Context;
use assyst_proc_macro::command;

use super::arguments::{Image, Rest, RestNoFlags, Word};
use super::flags::BloomFlags;
use crate::command::{Availability, Category, CommandCtxt};

#[command(
    description = "ah shit here we go again",
    cooldown = Duration::from_secs(2),
    access = Availability::Public,
    category = Category::Image,
    usage = "[image]",
    examples = ["https://link.to.my/image.png"],
    send_processing = true
)]
pub async fn ahshit(ctxt: CommandCtxt<'_>, source: Image) -> anyhow::Result<()> {
    let result = ctxt.flux_handler().ahshit(source.0, ctxt.data.author.id.get()).await?;

    ctxt.reply(result).await?;

    Ok(())
}

#[command(
    description = "april fools!!!!",
    cooldown = Duration::from_secs(2),
    access = Availability::Public,
    category = Category::Image,
    usage = "[image]",
    examples = ["https://link.to.my/image.png"],
    send_processing = true
)]
pub async fn aprilfools(ctxt: CommandCtxt<'_>, source: Image) -> anyhow::Result<()> {
    let result = ctxt
        .flux_handler()
        .aprilfools(source.0, ctxt.data.author.id.get())
        .await?;

    ctxt.reply(result).await?;

    Ok(())
}

#[command(
    name = "backtattoo",
    description = "put an image on a person's back",
    cooldown = Duration::from_secs(3),
    access = Availability::Public,
    category = Category::Image,
    usage = "[image]",
    examples = ["https://link.to.my/image.png"],
    send_processing = true
)]
pub async fn back_tattoo(ctxt: CommandCtxt<'_>, source: Image) -> anyhow::Result<()> {
    let result = ctxt
        .flux_handler()
        .back_tattoo(source.0, ctxt.data.author.id.get())
        .await?;

    ctxt.reply(result).await?;

    Ok(())
}

#[command(
    description = "put an image on a billboard",
    cooldown = Duration::from_secs(3),
    access = Availability::Public,
    category = Category::Image,
    usage = "[image]",
    examples = ["https://link.to.my/image.png"],
    send_processing = true
)]
pub async fn billboard(ctxt: CommandCtxt<'_>, source: Image) -> anyhow::Result<()> {
    let result = ctxt
        .flux_handler()
        .billboard(source.0, ctxt.data.author.id.get())
        .await?;

    ctxt.reply(result).await?;

    Ok(())
}

#[command(
    description = "bloom an image",
    cooldown = Duration::from_secs(2),
    access = Availability::Public,
    category = Category::Image,
    usage = "[image] <flags>",
    examples = ["https://link.to.my/image.png", "https://link.to.my/image.png --brightness 100 --sharpness 25 --radius 10"],
    send_processing = true,
    flag_descriptions = [
        ("radius", "Bloom radius as a number"),
        ("brightness", "Bloom brightness as a number"),
        ("sharpness", "Bloom sharpness as a number"),
    ]
)]
pub async fn bloom(ctxt: CommandCtxt<'_>, source: Image, flags: BloomFlags) -> anyhow::Result<()> {
    let result = ctxt
        .flux_handler()
        .bloom(
            source.0,
            flags.radius,
            flags.sharpness,
            flags.brightness,
            ctxt.data.author.id.get(),
        )
        .await?;

    ctxt.reply(result).await?;

    Ok(())
}

#[command(
    description = "blur an image",
    cooldown = Duration::from_secs(2),
    access = Availability::Public,
    category = Category::Image,
    usage = "[image] <radius>",
    examples = ["https://link.to.my/image.png"],
    send_processing = true
)]
pub async fn blur(ctxt: CommandCtxt<'_>, source: Image, strength: Option<f32>) -> anyhow::Result<()> {
    let result = ctxt
        .flux_handler()
        .blur(source.0, strength, ctxt.data.author.id.get())
        .await?;

    ctxt.reply(result).await?;

    Ok(())
}

#[command(
    description = "put an image in a book",
    cooldown = Duration::from_secs(3),
    access = Availability::Public,
    category = Category::Image,
    usage = "[image]",
    examples = ["https://link.to.my/image.png"],
    send_processing = true
)]
pub async fn book(ctxt: CommandCtxt<'_>, source: Image) -> anyhow::Result<()> {
    let result = ctxt.flux_handler().book(source.0, ctxt.data.author.id.get()).await?;

    ctxt.reply(result).await?;

    Ok(())
}

#[command(
    description = "add a caption to an image",
    cooldown = Duration::from_secs(2),
    access = Availability::Public,
    category = Category::Image,
    usage = "[image] [caption]",
    examples = ["https://link.to.my/image.png hello there"],
    send_processing = true
)]
pub async fn caption(ctxt: CommandCtxt<'_>, source: Image, text: Rest) -> anyhow::Result<()> {
    let result = ctxt
        .flux_handler()
        .caption(source.0, text.0, ctxt.data.author.id.get())
        .await?;

    ctxt.reply(result).await?;

    Ok(())
}

#[command(
    description = "put an image on a circuit board",
    cooldown = Duration::from_secs(3),
    access = Availability::Public,
    category = Category::Image,
    usage = "[image]",
    examples = ["https://link.to.my/image.png"],
    send_processing = true
)]
pub async fn circuitboard(ctxt: CommandCtxt<'_>, source: Image) -> anyhow::Result<()> {
    let result = ctxt
        .flux_handler()
        .circuitboard(source.0, ctxt.data.author.id.get())
        .await?;

    ctxt.reply(result).await?;

    Ok(())
}

#[command(
    description = "put an image on a flag",
    cooldown = Duration::from_secs(3),
    access = Availability::Public,
    category = Category::Image,
    usage = "[image]",
    examples = ["https://link.to.my/image.png"],
    send_processing = true
)]
pub async fn flag(ctxt: CommandCtxt<'_>, source: Image) -> anyhow::Result<()> {
    let result = ctxt.flux_handler().flag(source.0, ctxt.data.author.id.get()).await?;

    ctxt.reply(result).await?;

    Ok(())
}

#[command(
    description = "put an image on a different flag",
    cooldown = Duration::from_secs(3),
    access = Availability::Public,
    category = Category::Image,
    usage = "[image]",
    examples = ["https://link.to.my/image.png"],
    send_processing = true
)]
pub async fn flag2(ctxt: CommandCtxt<'_>, source: Image) -> anyhow::Result<()> {
    let result = ctxt.flux_handler().flag2(source.0, ctxt.data.author.id.get()).await?;

    ctxt.reply(result).await?;

    Ok(())
}

#[command(
    description = "put an image in a fortune cookie",
    cooldown = Duration::from_secs(3),
    access = Availability::Public,
    category = Category::Image,
    usage = "[image]",
    examples = ["https://link.to.my/image.png"],
    send_processing = true
)]
pub async fn fortunecookie(ctxt: CommandCtxt<'_>, source: Image) -> anyhow::Result<()> {
    let result = ctxt
        .flux_handler()
        .fortune_cookie(source.0, ctxt.data.author.id.get())
        .await?;

    ctxt.reply(result).await?;

    Ok(())
}

#[command(
    description = "put an image in a heart locket with some text",
    cooldown = Duration::from_secs(3),
    access = Availability::Public,
    category = Category::Image,
    usage = "[image]",
    examples = ["https://link.to.my/image.png"],
    send_processing = true
)]
pub async fn heartlocket(ctxt: CommandCtxt<'_>, source: Image, text: RestNoFlags) -> anyhow::Result<()> {
    let result = ctxt
        .flux_handler()
        .heart_locket(source.0, text.0, ctxt.data.author.id.get())
        .await?;

    ctxt.reply(result).await?;

    Ok(())
}

#[command(
    description = "resize an image based on scale or WxH (default 2x)",
    cooldown = Duration::from_secs(2),
    access = Availability::Public,
    category = Category::Image,
    usage = "[image] <scale>",
    examples = ["https://link.to.my/image.png", "https://link.to.my/image.png 128x128", "https://link.to.my/image.png 2"],
    send_processing = true
)]
pub async fn resize(ctxt: CommandCtxt<'_>, source: Image, size: Option<Word>) -> anyhow::Result<()> {
    let result = if let Some(ref i_size) = size
        && i_size.0.contains('x')
        && let Some((width, height)) = i_size.0.split_once('x')
    {
        let width = width.parse::<u32>().context("Invalid width")?;
        let height = height.parse::<u32>().context("Invalid height")?;

        ctxt.flux_handler()
            .resize_absolute(source.0, width, height, ctxt.data.author.id.get())
            .await?
    } else if let Some(i_size) = size {
        let scale = i_size.0.parse::<f32>().context("Invalid scale.")?;

        ctxt.flux_handler()
            .resize_scale(source.0, scale, ctxt.data.author.id.get())
            .await?
    } else {
        ctxt.flux_handler()
            .resize_scale(source.0, 2.0, ctxt.data.author.id.get())
            .await?
    };

    ctxt.reply(result).await?;

    Ok(())
}

#[command(
    description = "reverse a gif or video",
    cooldown = Duration::from_secs(3),
    access = Availability::Public,
    category = Category::Image,
    usage = "[image]",
    examples = ["https://link.to.my/image.png"],
    send_processing = true
)]
pub async fn reverse(ctxt: CommandCtxt<'_>, source: Image) -> anyhow::Result<()> {
    let result = ctxt.flux_handler().reverse(source.0, ctxt.data.author.id.get()).await?;

    ctxt.reply(result).await?;

    Ok(())
}

#[command(
    description = "put an image on a rubik's cube",
    cooldown = Duration::from_secs(3),
    access = Availability::Public,
    category = Category::Image,
    usage = "[image]",
    examples = ["https://link.to.my/image.png"],
    send_processing = true
)]
pub async fn rubiks(ctxt: CommandCtxt<'_>, source: Image) -> anyhow::Result<()> {
    let result = ctxt.flux_handler().rubiks(source.0, ctxt.data.author.id.get()).await?;

    ctxt.reply(result).await?;

    Ok(())
}

#[command(
    description = "put an image on a toaster",
    cooldown = Duration::from_secs(3),
    access = Availability::Public,
    category = Category::Image,
    usage = "[image]",
    examples = ["https://link.to.my/image.png"],
    send_processing = true
)]
pub async fn toaster(ctxt: CommandCtxt<'_>, source: Image) -> anyhow::Result<()> {
    let result = ctxt.flux_handler().toaster(source.0, ctxt.data.author.id.get()).await?;

    ctxt.reply(result).await?;

    Ok(())
}

#[command(
    description = "display an image as a valentine's gift",
    cooldown = Duration::from_secs(3),
    access = Availability::Public,
    category = Category::Image,
    usage = "[image]",
    examples = ["https://link.to.my/image.png"],
    send_processing = true
)]
pub async fn valentine(ctxt: CommandCtxt<'_>, source: Image) -> anyhow::Result<()> {
    let result = ctxt
        .flux_handler()
        .valentine(source.0, ctxt.data.author.id.get())
        .await?;

    ctxt.reply(result).await?;

    Ok(())
}
