use std::time::Duration;

use anyhow::Context;
use assyst_proc_macro::command;

use super::arguments::{Ge, Image, Ranged, Removable, Rest, Word};
use crate::command::{Availability, Category, CommandCtxt};

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

#[command(
    description = "create funny meme",
    cooldown = Duration::from_secs(4),
    access = Availability::Public,
    category = Category::Wsi,
    usage = "[image] [text separated by |]",
    examples = ["312715611413413889 this is|this is an otter"],
    send_processing = true
)]
pub async fn meme(_ctxt: CommandCtxt<'_>, _source: Image, _content: Rest) -> anyhow::Result<()> {
    anyhow::bail!("todo")
}

#[command(
    description = "apply a neon effect to an image",
    cooldown = Duration::from_secs(4),
    access = Availability::Public,
    category = Category::Wsi,
    usage = "[image] <power>",
    examples = ["312715611413413889", "312715611413413889 5"],
    send_processing = true
)]
pub async fn neon(_ctxt: CommandCtxt<'_>, _source: Image, _amount: Option<u64>) -> anyhow::Result<()> {
    anyhow::bail!("todo")
}

#[command(
    description = "prints an image forever",
    cooldown = Duration::from_secs(4),
    access = Availability::Public,
    category = Category::Wsi,
    usage = "[image]",
    examples = ["312715611413413889"],
    send_processing = true
)]
pub async fn printer(_ctxt: CommandCtxt<'_>, _source: Image) -> anyhow::Result<()> {
    anyhow::bail!("todo")
}

#[command(
    description = "apply motivational text to an image",
    cooldown = Duration::from_secs(4),
    access = Availability::Public,
    category = Category::Wsi,
    usage = "[image] [text separated by |]",
    examples = ["https://my.cool/png.png big text|small text"],
    send_processing = true
)]
pub async fn motivate(_ctxt: CommandCtxt<'_>, _source: Image, _content: Rest) -> anyhow::Result<()> {
    anyhow::bail!("todo")
}

#[command(
    description = "overlay an image onto another image",
    cooldown = Duration::from_secs(4),
    access = Availability::Public,
    category = Category::Wsi,
    usage = "[image] [text separated by |]",
    examples = ["312715611413413889 finland"],
    send_processing = true
)]
pub async fn overlay(_ctxt: CommandCtxt<'_>, _source: Image, _overlay: Word) -> anyhow::Result<()> {
    anyhow::bail!("todo")
}

#[command(
    description = "overlay (any) image onto another image",
    cooldown = Duration::from_secs(4),
    access = Availability::Public,
    category = Category::Wsi,
    usage = "[image] [image]",
    examples = ["312715611413413889 504698587221852172"],
    send_processing = true
)]
pub async fn overlay2(_ctxt: CommandCtxt<'_>, _source: Image, _overlay: Image) -> anyhow::Result<()> {
    anyhow::bail!("todo")
}

#[command(
    description = "paint an image",
    cooldown = Duration::from_secs(4),
    access = Availability::Public,
    category = Category::Wsi,
    usage = "[image]",
    examples = ["312715611413413889"],
    send_processing = true
)]
pub async fn paint(_ctxt: CommandCtxt<'_>, _source: Image) -> anyhow::Result<()> {
    anyhow::bail!("todo")
}

#[command(
    description = "pixelate an image",
    cooldown = Duration::from_secs(4),
    access = Availability::Public,
    category = Category::Wsi,
    usage = "[image]",
    examples = ["312715611413413889", "312715611413413889 4"],
    send_processing = true
)]
pub async fn pixelate(_ctxt: CommandCtxt<'_>, _source: Image, _pixels: Ge<0>) -> anyhow::Result<()> {
    anyhow::bail!("todo")
}

#[command(
    description = "rainbow-ify an image",
    cooldown = Duration::from_secs(4),
    access = Availability::Public,
    category = Category::Wsi,
    usage = "[image]",
    examples = ["312715611413413889"],
    send_processing = true
)]
pub async fn rainbow(_ctxt: CommandCtxt<'_>, _source: Image) -> anyhow::Result<()> {
    anyhow::bail!("todo")
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
    description = "reverse a gif",
    cooldown = Duration::from_secs(4),
    access = Availability::Public,
    category = Category::Wsi,
    usage = "[image]",
    examples = ["https://my.cool/gif.gif"],
    send_processing = true
)]
pub async fn reverse(_ctxt: CommandCtxt<'_>, _source: Image) -> anyhow::Result<()> {
    anyhow::bail!("todo")
}

#[command(
    description = "rotate an image",
    cooldown = Duration::from_secs(4),
    access = Availability::Public,
    category = Category::Wsi,
    usage = "[image]",
    examples = ["312715611413413889 45"],
    send_processing = true
)]
pub async fn rotate(_ctxt: CommandCtxt<'_>, _source: Image, _deg: Option<f64>) -> anyhow::Result<()> {
    anyhow::bail!("todo")
}

#[command(
    description = "configure whether a GIF will loop",
    cooldown = Duration::from_secs(4),
    access = Availability::Public,
    category = Category::Wsi,
    usage = "[image]",
    examples = ["https://my.cool/gif.gif false"],
    send_processing = true
)]
pub async fn setloop(_ctxt: CommandCtxt<'_>, _source: Image, _loop: bool) -> anyhow::Result<()> {
    anyhow::bail!("todo")
}

#[command(
    description = "add siren audio over an image",
    cooldown = Duration::from_secs(4),
    access = Availability::Public,
    category = Category::Wsi,
    usage = "[image]",
    examples = ["312715611413413889"],
    send_processing = true
)]
pub async fn siren(_ctxt: CommandCtxt<'_>, _source: Image) -> anyhow::Result<()> {
    anyhow::bail!("todo")
}

#[command(
    description = "add speechbubble over an image",
    aliases = ["bubble", "speech"],
    cooldown = Duration::from_secs(4),
    access = Availability::Public,
    category = Category::Wsi,
    usage = "[image]",
    examples = ["312715611413413889"],
    send_processing = true
)]
pub async fn speechbubble(_ctxt: CommandCtxt<'_>, _source: Image) -> anyhow::Result<()> {
    anyhow::bail!("todo")
}

#[command(
    description = "spin an image",
    cooldown = Duration::from_secs(4),
    access = Availability::Public,
    category = Category::Wsi,
    usage = "[image]",
    examples = ["312715611413413889"],
    send_processing = true
)]
pub async fn spin(_ctxt: CommandCtxt<'_>, _source: Image) -> anyhow::Result<()> {
    anyhow::bail!("todo")
}

// post yo money spread
#[command(
    description = "pixel-spread an image",
    cooldown = Duration::from_secs(4),
    access = Availability::Public,
    category = Category::Wsi,
    usage = "[image]",
    examples = ["312715611413413889"],
    send_processing = true
)]
pub async fn spread(_ctxt: CommandCtxt<'_>, _source: Image) -> anyhow::Result<()> {
    anyhow::bail!("todo")
}

#[command(
    description = "swirl an image",
    cooldown = Duration::from_secs(4),
    access = Availability::Public,
    category = Category::Wsi,
    usage = "[image]",
    examples = ["312715611413413889"],
    send_processing = true
)]
pub async fn swirl(_ctxt: CommandCtxt<'_>, _source: Image) -> anyhow::Result<()> {
    anyhow::bail!("todo")
}

#[command(
    description = "remove a caption from an image",
    cooldown = Duration::from_secs(4),
    access = Availability::Public,
    category = Category::Wsi,
    usage = "[image] <amount>",
    examples = ["312715611413413889 75", "312715611413413889 20%"],
    send_processing = true
)]
pub async fn uncaption(_ctxt: CommandCtxt<'_>, _source: Image, _amount: Word) -> anyhow::Result<()> {
    anyhow::bail!("todo")
}

#[command(
    description = "add minecraft music over an image",
    cooldown = Duration::from_secs(4),
    access = Availability::Public,
    category = Category::Wsi,
    usage = "[image]",
    examples = ["312715611413413889"],
    send_processing = true
)]
pub async fn sweden(_ctxt: CommandCtxt<'_>, _source: Image) -> anyhow::Result<()> {
    anyhow::bail!("todo")
}

#[command(
    description = "convert a video to a gif",
    aliases = ["vid2gif", "v2g", "togif"],
    cooldown = Duration::from_secs(4),
    access = Availability::Public,
    category = Category::Wsi,
    usage = "[image]",
    examples = ["https://my.cool/mp4.mp4"],
    send_processing = true
)]
pub async fn videotogif(_ctxt: CommandCtxt<'_>, _source: Image) -> anyhow::Result<()> {
    anyhow::bail!("todo")
}

#[command(
    description = "create a wall out of an image",
    cooldown = Duration::from_secs(4),
    access = Availability::Public,
    category = Category::Wsi,
    usage = "[image]",
    examples = ["312715611413413889"],
    send_processing = true
)]
pub async fn wall(_ctxt: CommandCtxt<'_>, _source: Image) -> anyhow::Result<()> {
    anyhow::bail!("todo")
}

#[command(
    description = "create a wave out of an image",
    cooldown = Duration::from_secs(4),
    access = Availability::Public,
    category = Category::Wsi,
    usage = "[image]",
    examples = ["312715611413413889"],
    send_processing = true
)]
pub async fn wave(_ctxt: CommandCtxt<'_>, _source: Image) -> anyhow::Result<()> {
    anyhow::bail!("todo")
}

#[command(
    description = "suck an image into a wormhole",
    cooldown = Duration::from_secs(4),
    access = Availability::Public,
    category = Category::Wsi,
    usage = "[image]",
    examples = ["312715611413413889"],
    send_processing = true
)]
pub async fn wormhole(_ctxt: CommandCtxt<'_>, _source: Image) -> anyhow::Result<()> {
    anyhow::bail!("todo")
}

#[command(
    description = "zoom into an image",
    cooldown = Duration::from_secs(4),
    access = Availability::Public,
    category = Category::Wsi,
    usage = "[image]",
    examples = ["312715611413413889"],
    send_processing = true
)]
pub async fn zoom(_ctxt: CommandCtxt<'_>, _source: Image) -> anyhow::Result<()> {
    anyhow::bail!("todo")
}
