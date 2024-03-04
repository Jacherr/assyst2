use std::time::Duration;

use anyhow::Context;
use assyst_proc_macro::command;

use super::arguments::{Image, Ranged, Removable, Rest, Word};
use crate::command::{Availability, Category, CommandCtxt};

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

#[command(
    description = "ah shit, here we go again",
    cooldown = Duration::from_secs(4),
    access = Availability::Public,
    category = Category::Wsi,
    usage = "[image]",
    examples = ["312715611413413889"],
    send_processing = true
)]
pub async fn ahshit(_ctxt: CommandCtxt<'_>, _source: Image) -> anyhow::Result<()> {
    anyhow::bail!("todo")
}

#[command(
    description = "april fools",
    cooldown = Duration::from_secs(4),
    access = Availability::Public,
    category = Category::Wsi,
    usage = "[image]",
    examples = ["312715611413413889"],
    send_processing = true
)]
pub async fn aprilfools(_ctxt: CommandCtxt<'_>, _source: Image) -> anyhow::Result<()> {
    anyhow::bail!("todo")
}

#[command(
    description = "bloom an image",
    aliases = ["softglow"],
    cooldown = Duration::from_secs(4),
    access = Availability::Public,
    category = Category::Wsi,
    usage = "[image] <radius> <brightness> <sharpness>",
    examples = ["312715611413413889", "312715611413413889 5", "312715611413413889 _ 30", "312715611413413889 10 30 85"],
    send_processing = true
)]
pub async fn bloom(
    _ctxt: CommandCtxt<'_>,
    _source: Image,
    _radius: Removable<f64>,
    _brightness: Removable<f64>,
    _sharpness: Removable<f64>,
) -> anyhow::Result<()> {
    anyhow::bail!("todo")
}

#[command(
    description = "blur",
    cooldown = Duration::from_secs(4),
    access = Availability::Public,
    category = Category::Wsi,
    usage = "[image]",
    examples = ["312715611413413889"],
    send_processing = true
)]
pub async fn blur(_ctxt: CommandCtxt<'_>, _source: Image, _power: Option<f64>) -> anyhow::Result<()> {
    anyhow::bail!("todo")
}

#[command(
    description = "create burning text",
    cooldown = Duration::from_secs(4),
    access = Availability::Public,
    category = Category::Wsi,
    usage = "[text]",
    examples = ["this text is burning"],
    send_processing = true
)]
pub async fn burntext(_ctxt: CommandCtxt<'_>, _source: Rest) -> anyhow::Result<()> {
    anyhow::bail!("todo")
}

#[command(
    description = "deep fry an image",
    aliases = ["df"],
    cooldown = Duration::from_secs(4),
    access = Availability::Public,
    category = Category::Wsi,
    usage = "[image]",
    examples = ["312715611413413889"],
    send_processing = true
)]
pub async fn deepfry(_ctxt: CommandCtxt<'_>, _source: Image) -> anyhow::Result<()> {
    anyhow::bail!("todo")
}

#[command(
    description = "among us drip music over image",
    cooldown = Duration::from_secs(4),
    access = Availability::Public,
    category = Category::Wsi,
    usage = "[image]",
    examples = ["312715611413413889"],
    send_processing = true
)]
pub async fn drip(_ctxt: CommandCtxt<'_>, _source: Image) -> anyhow::Result<()> {
    anyhow::bail!("todo")
}

#[command(
    description = "flip an image (vertical)",
    cooldown = Duration::from_secs(4),
    access = Availability::Public,
    category = Category::Wsi,
    usage = "[image]",
    examples = ["312715611413413889"],
    send_processing = true
)]
pub async fn flip(_ctxt: CommandCtxt<'_>, _source: Image) -> anyhow::Result<()> {
    anyhow::bail!("todo")
}

#[command(
    description = "flop an image (horizontal)",
    cooldown = Duration::from_secs(4),
    access = Availability::Public,
    category = Category::Wsi,
    usage = "[image]",
    examples = ["312715611413413889"],
    send_processing = true
)]
pub async fn flop(_ctxt: CommandCtxt<'_>, _source: Image) -> anyhow::Result<()> {
    anyhow::bail!("todo")
}

#[command(
    description = "frameshift an image",
    aliases = ["butt", "fshift"],
    cooldown = Duration::from_secs(4),
    access = Availability::Public,
    category = Category::Wsi,
    usage = "[image]",
    examples = ["312715611413413889"],
    send_processing = true
)]
pub async fn frameshift(_ctxt: CommandCtxt<'_>, _source: Image) -> anyhow::Result<()> {
    anyhow::bail!("todo")
}

#[command(
    description = "fisheye an image",
    aliases = ["fish"],
    cooldown = Duration::from_secs(4),
    access = Availability::Public,
    category = Category::Wsi,
    usage = "[image]",
    examples = ["312715611413413889"],
    send_processing = true
)]
pub async fn fisheye(_ctxt: CommandCtxt<'_>, _source: Image) -> anyhow::Result<()> {
    anyhow::bail!("todo")
}

#[command(
    description = "get frames of a gif",
    cooldown = Duration::from_secs(4),
    access = Availability::Public,
    category = Category::Wsi,
    usage = "[image]",
    examples = ["312715611413413889"],
    send_processing = true
)]
pub async fn frames(_ctxt: CommandCtxt<'_>, _source: Image) -> anyhow::Result<()> {
    anyhow::bail!("todo")
}

#[command(
    description = "femurbreaker over image (loud)",
    cooldown = Duration::from_secs(4),
    access = Availability::Public,
    category = Category::Wsi,
    usage = "[image]",
    examples = ["312715611413413889"],
    send_processing = true
)]
pub async fn femurbreaker(_ctxt: CommandCtxt<'_>, _source: Image) -> anyhow::Result<()> {
    anyhow::bail!("todo")
}

#[command(
    description = "perform frame ghosting on a gif",
    cooldown = Duration::from_secs(4),
    access = Availability::Public,
    category = Category::Wsi,
    usage = "[image] <frames>",
    examples = ["https://my.cool/gif.gif"],
    send_processing = true
)]
pub async fn ghost(_ctxt: CommandCtxt<'_>, _source: Image, _frames: Option<Ranged<10, 20>>) -> anyhow::Result<()> {
    anyhow::bail!("todo")
}

#[command(
    description = "play a gif forward and then backward",
    aliases = ["gloop"],
    cooldown = Duration::from_secs(4),
    access = Availability::Public,
    category = Category::Wsi,
    usage = "[image]",
    examples = ["https://my.cool/gif.gif"],
    send_processing = true
)]
pub async fn gifloop(_ctxt: CommandCtxt<'_>, _source: Image) -> anyhow::Result<()> {
    anyhow::bail!("todo")
}

#[command(
    description = "scramble the frames in a gif",
    aliases = ["gscramble"],
    cooldown = Duration::from_secs(4),
    access = Availability::Public,
    category = Category::Wsi,
    usage = "[image]",
    examples = ["https://my.cool/gif.gif"],
    send_processing = true
)]
pub async fn gifscramble(_ctxt: CommandCtxt<'_>, _source: Image) -> anyhow::Result<()> {
    anyhow::bail!("todo")
}

#[command(
    description = "alter the speed of a gif",
    aliases = ["gspeed", "speed"],
    cooldown = Duration::from_secs(4),
    access = Availability::Public,
    category = Category::Wsi,
    usage = "[image] <speed>",
    examples = ["https://my.cool/gif.gif"],
    send_processing = true
)]
pub async fn gifspeed(_ctxt: CommandCtxt<'_>, _source: Image, _speed: f64) -> anyhow::Result<()> {
    anyhow::bail!("todo")
}

#[command(
    description = "turn an image into a spinning globe",
    aliases = ["sphere"],
    cooldown = Duration::from_secs(4),
    access = Availability::Public,
    category = Category::Wsi,
    usage = "[image]",
    examples = ["312715611413413889"],
    send_processing = true
)]
pub async fn globe(_ctxt: CommandCtxt<'_>, _source: Image) -> anyhow::Result<()> {
    anyhow::bail!("todo")
}

#[command(
    description = "grayscale an image",
    aliases = ["gray", "greyscale", "grayscale"],
    cooldown = Duration::from_secs(4),
    access = Availability::Public,
    category = Category::Wsi,
    usage = "[image]",
    examples = ["312715611413413889"],
    send_processing = true
)]
pub async fn grayscale(_ctxt: CommandCtxt<'_>, _source: Image) -> anyhow::Result<()> {
    anyhow::bail!("todo")
}

#[command(
    description = "get information about an image",
    aliases = ["ii", "exif"],
    cooldown = Duration::from_secs(4),
    access = Availability::Public,
    category = Category::Wsi,
    usage = "[image]",
    examples = ["312715611413413889"],
    send_processing = true
)]
pub async fn image_info(_ctxt: CommandCtxt<'_>, _source: Image) -> anyhow::Result<()> {
    anyhow::bail!("todo")
}

#[command(
    description = "evaluate an imagemagick script on an image",
    aliases = ["ime"],
    cooldown = Duration::from_secs(4),
    access = Availability::Dev,
    category = Category::Wsi,
    usage = "[image] [command]",
    examples = ["312715611413413889"],
    send_processing = true
)]
pub async fn imagemagick_eval(_ctxt: CommandCtxt<'_>, _source: Image, _cmd: Rest) -> anyhow::Result<()> {
    anyhow::bail!("todo")
}

#[command(
    description = "invert an image",
    cooldown = Duration::from_secs(4),
    access = Availability::Public,
    category = Category::Wsi,
    usage = "[image]",
    examples = ["312715611413413889"],
    send_processing = true
)]
pub async fn invert(_ctxt: CommandCtxt<'_>, _source: Image) -> anyhow::Result<()> {
    anyhow::bail!("todo")
}

#[command(
    description = "JPEG-ify an image",
    cooldown = Duration::from_secs(4),
    access = Availability::Public,
    category = Category::Wsi,
    usage = "[image]",
    examples = ["312715611413413889"],
    send_processing = true
)]
pub async fn jpeg(_ctxt: CommandCtxt<'_>, _source: Image) -> anyhow::Result<()> {
    anyhow::bail!("todo")
}

#[command(
    description = "perform seam carving on an image",
    aliases = ["magick", "cas", "magic"],
    cooldown = Duration::from_secs(4),
    access = Availability::Public,
    category = Category::Wsi,
    usage = "[image]",
    examples = ["312715611413413889"],
    send_processing = true
)]
pub async fn magik(_ctxt: CommandCtxt<'_>, _source: Image) -> anyhow::Result<()> {
    anyhow::bail!("todo")
}
