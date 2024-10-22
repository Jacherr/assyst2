use std::time::Duration;

use anyhow::Context;
use assyst_common::util::{format_duration, table};
use assyst_flux_iface::jobs::MediaInfo;
use assyst_proc_macro::command;
use assyst_string_fmt::{Ansi, Markdown};
use human_bytes::human_bytes;

use super::arguments::{Image, RestNoFlags, Word};
use super::messagebuilder::{Attachment, MessageBuilder};
use crate::command::{Availability, Category, CommandCtxt};

pub mod audio;
pub mod bloom;
pub mod caption;
pub mod makesweet;
pub mod randomize;
pub mod speechbubble;

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
    let result = ctxt
        .flux_handler()
        .ahshit(source.0, ctxt.data.author.id.get(), ctxt.data.guild_id.map(twilight_model::id::Id::get))
        .await?;

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
        .aprilfools(source.0, ctxt.data.author.id.get(), ctxt.data.guild_id.map(twilight_model::id::Id::get))
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
        .blur(
            source.0,
            strength,
            ctxt.data.author.id.get(),
            ctxt.data.guild_id.map(twilight_model::id::Id::get),
        )
        .await?;

    ctxt.reply(result).await?;

    Ok(())
}

#[command(
    description = "deep fry an image",
    aliases = ["df"],
    cooldown = Duration::from_secs(2),
    access = Availability::Public,
    category = Category::Image,
    usage = "[image]",
    examples = ["https://link.to.my/image.png"],
    send_processing = true
)]
pub async fn deepfry(ctxt: CommandCtxt<'_>, source: Image) -> anyhow::Result<()> {
    let result = ctxt
        .flux_handler()
        .deepfry(source.0, ctxt.data.author.id.get(), ctxt.data.guild_id.map(twilight_model::id::Id::get))
        .await?;

    ctxt.reply(result).await?;

    Ok(())
}

#[command(
    description = "fish-eye an image",
    cooldown = Duration::from_secs(2),
    access = Availability::Public,
    category = Category::Image,
    usage = "[image]",
    examples = ["https://link.to.my/image.png"],
    send_processing = true
)]
pub async fn fisheye(ctxt: CommandCtxt<'_>, source: Image) -> anyhow::Result<()> {
    let result = ctxt
        .flux_handler()
        .fisheye(source.0, ctxt.data.author.id.get(), ctxt.data.guild_id.map(twilight_model::id::Id::get))
        .await?;

    ctxt.reply(result).await?;

    Ok(())
}

#[command(
    description = "flip an image vertically",
    cooldown = Duration::from_secs(2),
    access = Availability::Public,
    category = Category::Image,
    usage = "[image]",
    examples = ["https://link.to.my/image.png"],
    send_processing = true
)]
pub async fn flip(ctxt: CommandCtxt<'_>, source: Image) -> anyhow::Result<()> {
    let result = ctxt
        .flux_handler()
        .flip(source.0, ctxt.data.author.id.get(), ctxt.data.guild_id.map(twilight_model::id::Id::get))
        .await?;

    ctxt.reply(result).await?;

    Ok(())
}

#[command(
    description = "flop an image horizontally",
    cooldown = Duration::from_secs(2),
    access = Availability::Public,
    category = Category::Image,
    usage = "[image]",
    examples = ["https://link.to.my/image.png"],
    send_processing = true
)]
pub async fn flop(ctxt: CommandCtxt<'_>, source: Image) -> anyhow::Result<()> {
    let result = ctxt
        .flux_handler()
        .flop(source.0, ctxt.data.author.id.get(), ctxt.data.guild_id.map(twilight_model::id::Id::get))
        .await?;

    ctxt.reply(result).await?;

    Ok(())
}

#[command(
    description = "extract all frames of a gif or video and zip them",
    cooldown = Duration::from_secs(2),
    access = Availability::Public,
    category = Category::Image,
    usage = "[image]",
    examples = ["https://link.to.my/image.png"],
    send_processing = true
)]
pub async fn frames(ctxt: CommandCtxt<'_>, source: Image) -> anyhow::Result<()> {
    let result = ctxt
        .flux_handler()
        .frames(source.0, ctxt.data.author.id.get(), ctxt.data.guild_id.map(twilight_model::id::Id::get))
        .await?;

    let response: MessageBuilder = MessageBuilder {
        content: None,
        attachment: Some(Attachment {
            name: "frames.zip".to_owned().into_boxed_str(),
            data: result,
        }),
        component_ctxt: None,
        components: None,
    };

    ctxt.reply(response).await?;

    Ok(())
}

#[command(
    description = "perform some frame shifting (please suggest a better name for this command)",
    cooldown = Duration::from_secs(2),
    access = Availability::Public,
    category = Category::Image,
    usage = "[image]",
    examples = ["https://link.to.my/image.png"],
    send_processing = true
)]
pub async fn frameshift(ctxt: CommandCtxt<'_>, source: Image) -> anyhow::Result<()> {
    let result = ctxt
        .flux_handler()
        .frame_shift(source.0, ctxt.data.author.id.get(), ctxt.data.guild_id.map(twilight_model::id::Id::get))
        .await?;

    ctxt.reply(result).await?;

    Ok(())
}

#[command(
    description = "add a ghosting effect to a gif or video",
    cooldown = Duration::from_secs(3),
    access = Availability::Public,
    category = Category::Image,
    usage = "[image] <depth>",
    examples = ["https://link.to.my/image.png", "https://link.to.my/image.png 10"],
    send_processing = true
)]
pub async fn ghost(ctxt: CommandCtxt<'_>, source: Image, depth: Option<u64>) -> anyhow::Result<()> {
    let result = ctxt
        .flux_handler()
        .ghost(
            source.0,
            depth,
            ctxt.data.author.id.get(),
            ctxt.data.guild_id.map(twilight_model::id::Id::get),
        )
        .await?;

    ctxt.reply(result).await?;

    Ok(())
}

#[command(
    description = "convert an input to a gif",
    cooldown = Duration::from_secs(4),
    access = Availability::Public,
    category = Category::Image,
    usage = "[image]",
    examples = ["https://link.to.my/image.png"],
    send_processing = true
)]
pub async fn gif(ctxt: CommandCtxt<'_>, source: Image) -> anyhow::Result<()> {
    let result = ctxt
        .flux_handler()
        .gif(source.0, ctxt.data.author.id.get(), ctxt.data.guild_id.map(twilight_model::id::Id::get))
        .await?;

    ctxt.reply(result).await?;

    Ok(())
}

#[command(
    description = "create a magikal gif out of an image",
    aliases = ["gifmagic", "gifmagick", "gmagik", "gifcas", "gcas"],
    cooldown = Duration::from_secs(6),
    access = Availability::Public,
    category = Category::Image,
    usage = "[image]",
    examples = ["https://link.to.my/image.png"],
    send_processing = true
)]
pub async fn gifmagik(ctxt: CommandCtxt<'_>, source: Image) -> anyhow::Result<()> {
    let result = ctxt
        .flux_handler()
        .gif_magik(source.0, ctxt.data.author.id.get(), ctxt.data.guild_id.map(twilight_model::id::Id::get))
        .await?;

    ctxt.reply(result).await?;

    Ok(())
}

#[command(
    description = "turn an image into a globe",
    cooldown = Duration::from_secs(4),
    access = Availability::Public,
    category = Category::Image,
    usage = "[image]",
    examples = ["https://link.to.my/image.png"],
    send_processing = true
)]
pub async fn globe(ctxt: CommandCtxt<'_>, source: Image) -> anyhow::Result<()> {
    let result = ctxt
        .flux_handler()
        .globe(source.0, ctxt.data.author.id.get(), ctxt.data.guild_id.map(twilight_model::id::Id::get))
        .await?;

    ctxt.reply(result).await?;

    Ok(())
}

#[command(
    description = "grayscale an image",
    aliases = ["gray", "greyscale", "grey"],
    cooldown = Duration::from_secs(2),
    access = Availability::Public,
    category = Category::Image,
    usage = "[image]",
    examples = ["https://link.to.my/image.png"],
    send_processing = true
)]
pub async fn grayscale(ctxt: CommandCtxt<'_>, source: Image) -> anyhow::Result<()> {
    let result = ctxt
        .flux_handler()
        .grayscale(source.0, ctxt.data.author.id.get(), ctxt.data.guild_id.map(twilight_model::id::Id::get))
        .await?;

    ctxt.reply(result).await?;

    Ok(())
}

#[command(
    description = "get info about an image or video",
    aliases = ["ii", "videoinfo", "exif", "vi"],
    cooldown = Duration::from_secs(2),
    access = Availability::Public,
    category = Category::Image,
    usage = "[image]",
    examples = ["https://link.to.my/image.png"],
    send_processing = true,
    context_menu_message_command = "Image Information"
)]
pub async fn imageinfo(ctxt: CommandCtxt<'_>, source: Image) -> anyhow::Result<()> {
    let result = ctxt.flux_handler().image_info(source.0).await?;

    if let MediaInfo::Image(i) = result {
        let mut kv = Vec::new();
        kv.push(("Filesize".fg_cyan(), human_bytes(i.file_size_bytes as f64)));
        kv.push(("Mimetype".fg_cyan(), i.mime_type));
        kv.push(("Dimensions".fg_cyan(), i.dimensions));
        if let Some(count) = i.frame_count {
            kv.push(("Frame Count".fg_cyan(), count.to_string()));
        };
        if let Some(r) = i.repeat {
            kv.push(("Repeat".fg_cyan(), r));
        };
        if !i.comments.is_empty() {
            kv.push((
                "Comments".fg_cyan(),
                i.comments
                    .iter()
                    .enumerate()
                    .map(|(i, c)| format!("{i}: {c}"))
                    .collect::<Vec<_>>()
                    .join(", "),
            ));
        };
        let table = table::key_value(&kv);
        ctxt.reply(table.codeblock("ansi")).await?;
    } else if let MediaInfo::Video(v) = result {
        let kv = vec![
            ("Filesize".fg_cyan(), human_bytes(v.file_size_bytes as f64)),
            ("Mimetype".fg_cyan(), v.mime_type),
            ("Dimensions".fg_cyan(), v.dimensions),
            (
                "Duration".fg_cyan(),
                format_duration(&Duration::from_millis(v.duration_ms)),
            ),
            ("Frame Count".fg_cyan(), v.frame_count.to_string()),
            ("Frames Per Second".fg_cyan(), v.fps.to_string()),
        ];
        let table = table::key_value(&kv);
        ctxt.reply(table.codeblock("ansi")).await?;
    }

    Ok(())
}

#[command(
    description = "invert an image",
    aliases = ["negate"],
    cooldown = Duration::from_secs(2),
    access = Availability::Public,
    category = Category::Image,
    usage = "[image]",
    examples = ["https://link.to.my/image.png"],
    send_processing = true
)]
pub async fn invert(ctxt: CommandCtxt<'_>, source: Image) -> anyhow::Result<()> {
    let result = ctxt
        .flux_handler()
        .invert(source.0, ctxt.data.author.id.get(), ctxt.data.guild_id.map(twilight_model::id::Id::get))
        .await?;

    ctxt.reply(result).await?;

    Ok(())
}

#[command(
    description = "jpeg-ify an image",
    aliases = ["jpg"],
    cooldown = Duration::from_secs(2),
    access = Availability::Public,
    category = Category::Image,
    usage = "[image] <quality: 1-100>",
    examples = ["https://link.to.my/image.png 5"],
    send_processing = true
)]
pub async fn jpeg(ctxt: CommandCtxt<'_>, source: Image, quality: Option<u64>) -> anyhow::Result<()> {
    let result = ctxt
        .flux_handler()
        .jpeg(
            source.0,
            quality,
            ctxt.data.author.id.get(),
            ctxt.data.guild_id.map(twilight_model::id::Id::get),
        )
        .await?;

    ctxt.reply(result).await?;

    Ok(())
}

#[command(
    description = "give your input some magik",
    aliases = ["magic", "magick", "cas"],
    cooldown = Duration::from_secs(4),
    access = Availability::Public,
    category = Category::Image,
    usage = "[image]",
    examples = ["https://link.to.my/image.png"],
    send_processing = true
)]
pub async fn magik(ctxt: CommandCtxt<'_>, source: Image) -> anyhow::Result<()> {
    let result = ctxt
        .flux_handler()
        .magik(source.0, ctxt.data.author.id.get(), ctxt.data.guild_id.map(twilight_model::id::Id::get))
        .await?;

    ctxt.reply(result).await?;

    Ok(())
}

#[command(
    description = "add impact font meme text to an image",
    cooldown = Duration::from_secs(3),
    access = Availability::Public,
    category = Category::Image,
    usage = "[image] [text separated by |]",
    examples = ["https://link.to.my/image.png hello there you", "https://link.to.my/image.png |hello there you", "https://link.to.my/image.png hello there|you"],
    send_processing = true
)]
pub async fn meme(ctxt: CommandCtxt<'_>, source: Image, text: RestNoFlags) -> anyhow::Result<()> {
    let text = text.0;

    let divider = if text.contains('|') {
        "|".to_string()
    } else {
        " ".to_string()
    };

    let parts = text
        .split_once(&divider)
        .map(|(x, y)| (Some(x.to_owned()), Some(y.to_owned())))
        .unwrap_or((Some(text.clone()), None));

    let top_text = parts.0;
    let bottom_text = parts.1;

    let result = ctxt
        .flux_handler()
        .meme(
            source.0,
            top_text,
            bottom_text,
            ctxt.data.author.id.get(),
            ctxt.data.guild_id.map(twilight_model::id::Id::get),
        )
        .await?;

    ctxt.reply(result).await?;

    Ok(())
}

#[command(
    description = "create a motivational poster out of an image",
    cooldown = Duration::from_secs(3),
    access = Availability::Public,
    category = Category::Image,
    usage = "[image] [text separated by |]",
    examples = ["https://link.to.my/image.png hello there you", "https://link.to.my/image.png |hello there you", "https://link.to.my/image.png hello there|you"],
    send_processing = true
)]
pub async fn motivate(ctxt: CommandCtxt<'_>, source: Image, text: RestNoFlags) -> anyhow::Result<()> {
    let text = text.0;

    let divider = if text.contains('|') {
        "|".to_string()
    } else {
        " ".to_string()
    };

    let parts = text
        .split_once(&divider)
        .map(|(x, y)| (Some(x.to_owned()), Some(y.to_owned())))
        .unwrap_or((Some(text.clone()), None));

    let top_text = parts.0;
    let bottom_text = parts.1;

    let result = ctxt
        .flux_handler()
        .motivate(
            source.0,
            top_text,
            bottom_text,
            ctxt.data.author.id.get(),
            ctxt.data.guild_id.map(twilight_model::id::Id::get),
        )
        .await?;

    ctxt.reply(result).await?;

    Ok(())
}

#[command(
    description = "apply a neon effect to an image",
    aliases = ["negate"],
    cooldown = Duration::from_secs(2),
    access = Availability::Public,
    category = Category::Image,
    usage = "[image]",
    examples = ["https://link.to.my/image.png"],
    send_processing = true
)]
pub async fn neon(ctxt: CommandCtxt<'_>, source: Image) -> anyhow::Result<()> {
    let result = ctxt
        .flux_handler()
        .neon(source.0, ctxt.data.author.id.get(), ctxt.data.guild_id.map(twilight_model::id::Id::get))
        .await?;

    ctxt.reply(result).await?;

    Ok(())
}

#[command(
    description = "overlay one image over another",
    cooldown = Duration::from_secs(2),
    access = Availability::Public,
    category = Category::Image,
    usage = "[image]",
    examples = ["https://link.to.my/image.png"],
    send_processing = true
)]
pub async fn overlay(ctxt: CommandCtxt<'_>, source: Image, source2: Image) -> anyhow::Result<()> {
    let result = ctxt
        .flux_handler()
        .overlay(
            source.0,
            source2.0,
            ctxt.data.author.id.get(),
            ctxt.data.guild_id.map(twilight_model::id::Id::get),
        )
        .await?;

    ctxt.reply(result).await?;

    Ok(())
}

#[command(
    description = "paint an image",
    cooldown = Duration::from_secs(4),
    access = Availability::Public,
    category = Category::Image,
    usage = "[image]",
    examples = ["https://link.to.my/image.png"],
    send_processing = true
)]
pub async fn paint(ctxt: CommandCtxt<'_>, source: Image) -> anyhow::Result<()> {
    let result = ctxt
        .flux_handler()
        .paint(source.0, ctxt.data.author.id.get(), ctxt.data.guild_id.map(twilight_model::id::Id::get))
        .await?;

    ctxt.reply(result).await?;

    Ok(())
}

#[command(
    description = "play a gif forward then backward",
    aliases = ["gifloop", "gloop"],
    cooldown = Duration::from_secs(4),
    access = Availability::Public,
    category = Category::Image,
    usage = "[image]",
    examples = ["https://link.to.my/image.png"],
    send_processing = true
)]
pub async fn pingpong(ctxt: CommandCtxt<'_>, source: Image) -> anyhow::Result<()> {
    let result = ctxt
        .flux_handler()
        .ping_pong(source.0, ctxt.data.author.id.get(), ctxt.data.guild_id.map(twilight_model::id::Id::get))
        .await?;

    ctxt.reply(result).await?;

    Ok(())
}

#[command(
    description = "pixelate an image",
    cooldown = Duration::from_secs(2),
    access = Availability::Public,
    category = Category::Image,
    usage = "[image] <strength: 1-100>",
    examples = ["https://link.to.my/image.png 5"],
    send_processing = true
)]
pub async fn pixelate(ctxt: CommandCtxt<'_>, source: Image, strength: Option<f32>) -> anyhow::Result<()> {
    let result = ctxt
        .flux_handler()
        .pixelate(
            source.0,
            strength,
            ctxt.data.author.id.get(),
            ctxt.data.guild_id.map(twilight_model::id::Id::get),
        )
        .await?;

    ctxt.reply(result).await?;

    Ok(())
}

#[command(
    description = "make an image rainbow",
    cooldown = Duration::from_secs(4),
    access = Availability::Public,
    category = Category::Image,
    usage = "[image]",
    examples = ["https://link.to.my/image.png"],
    send_processing = true
)]
pub async fn rainbow(ctxt: CommandCtxt<'_>, source: Image) -> anyhow::Result<()> {
    let result = ctxt
        .flux_handler()
        .rainbow(source.0, ctxt.data.author.id.get(), ctxt.data.guild_id.map(twilight_model::id::Id::get))
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
            .resize_absolute(
                source.0,
                width,
                height,
                ctxt.data.author.id.get(),
                ctxt.data.guild_id.map(twilight_model::id::Id::get),
            )
            .await?
    } else if let Some(i_size) = size {
        let scale = i_size.0.parse::<f32>().context("Invalid scale.")?;

        ctxt.flux_handler()
            .resize_scale(
                source.0,
                scale,
                ctxt.data.author.id.get(),
                ctxt.data.guild_id.map(twilight_model::id::Id::get),
            )
            .await?
    } else {
        ctxt.flux_handler()
            .resize_scale(
                source.0,
                2.0,
                ctxt.data.author.id.get(),
                ctxt.data.guild_id.map(twilight_model::id::Id::get),
            )
            .await?
    };

    ctxt.reply(result).await?;

    Ok(())
}

#[command(
    description = "reverse a gif or video",
    cooldown = Duration::from_secs(2),
    access = Availability::Public,
    category = Category::Image,
    usage = "[image]",
    examples = ["https://link.to.my/image.png"],
    send_processing = true
)]
pub async fn reverse(ctxt: CommandCtxt<'_>, source: Image) -> anyhow::Result<()> {
    let result = ctxt
        .flux_handler()
        .reverse(source.0, ctxt.data.author.id.get(), ctxt.data.guild_id.map(twilight_model::id::Id::get))
        .await?;

    ctxt.reply(result).await?;

    Ok(())
}

#[command(
    description = "rotate an image",
    cooldown = Duration::from_secs(2),
    access = Availability::Public,
    category = Category::Image,
    usage = "[image] <degrees: 0-360>",
    examples = ["https://link.to.my/image.png 45"],
    send_processing = true
)]
pub async fn rotate(ctxt: CommandCtxt<'_>, source: Image, degrees: Option<u64>) -> anyhow::Result<()> {
    let result = ctxt
        .flux_handler()
        .rotate(
            source.0,
            degrees,
            ctxt.data.author.id.get(),
            ctxt.data.guild_id.map(twilight_model::id::Id::get),
        )
        .await?;

    ctxt.reply(result).await?;

    Ok(())
}

#[command(
    description = "scramble a gif or video",
    aliases = ["gifscramble", "gscramble"],
    cooldown = Duration::from_secs(6),
    access = Availability::Public,
    category = Category::Image,
    usage = "[image]",
    examples = ["https://link.to.my/image.png"],
    send_processing = true
)]
pub async fn scramble(ctxt: CommandCtxt<'_>, source: Image) -> anyhow::Result<()> {
    let result = ctxt
        .flux_handler()
        .scramble(source.0, ctxt.data.author.id.get(), ctxt.data.guild_id.map(twilight_model::id::Id::get))
        .await?;

    ctxt.reply(result).await?;

    Ok(())
}

#[command(
    description = "set how many times a gif loops",
    aliases = ["loop"],
    cooldown = Duration::from_secs(6),
    access = Availability::Public,
    category = Category::Image,
    usage = "[image] [loop count (-1 for infinite)]",
    examples = ["https://link.to.my/image.png 3", "https://link.to.my/image.png -1"],
    send_processing = true
)]
pub async fn setloop(ctxt: CommandCtxt<'_>, source: Image, loops: i64) -> anyhow::Result<()> {
    let result = ctxt
        .flux_handler()
        .set_loop(
            source.0,
            ctxt.data.author.id.get(),
            ctxt.data.guild_id.map(twilight_model::id::Id::get),
            loops,
        )
        .await?;

    ctxt.reply(result).await?;

    Ok(())
}

#[command(
    description = "speed up or slow down a gif or video",
    aliases = ["gspeed", "gifspeed"],
    cooldown = Duration::from_secs(3),
    access = Availability::Public,
    category = Category::Image,
    usage = "[image] <multiplier>",
    examples = ["https://link.to.my/image.png", "https://link.to.my/image.png 3"],
    send_processing = true
)]
pub async fn speed(ctxt: CommandCtxt<'_>, source: Image, multiplier: Option<f64>) -> anyhow::Result<()> {
    let result = ctxt
        .flux_handler()
        .speed(
            source.0,
            multiplier,
            ctxt.data.author.id.get(),
            ctxt.data.guild_id.map(twilight_model::id::Id::get),
        )
        .await?;

    ctxt.reply(result).await?;

    Ok(())
}

#[command(
    description = "spin an image",
    cooldown = Duration::from_secs(3),
    access = Availability::Public,
    category = Category::Image,
    usage = "[image]",
    examples = ["https://link.to.my/image.png"],
    send_processing = true
)]
pub async fn spin(ctxt: CommandCtxt<'_>, source: Image) -> anyhow::Result<()> {
    let result = ctxt
        .flux_handler()
        .spin(source.0, ctxt.data.author.id.get(), ctxt.data.guild_id.map(twilight_model::id::Id::get))
        .await?;

    ctxt.reply(result).await?;

    Ok(())
}

#[command(
    description = "pixel-spread an image",
    cooldown = Duration::from_secs(3),
    access = Availability::Public,
    category = Category::Image,
    usage = "[image] <strength>",
    examples = ["https://link.to.my/image.png", "https://link.to.my/image.png 15"],
    send_processing = true
)]
pub async fn spread(ctxt: CommandCtxt<'_>, source: Image, strength: Option<u64>) -> anyhow::Result<()> {
    let result = ctxt
        .flux_handler()
        .spread(
            source.0,
            strength,
            ctxt.data.author.id.get(),
            ctxt.data.guild_id.map(twilight_model::id::Id::get),
        )
        .await?;

    ctxt.reply(result).await?;

    Ok(())
}

#[command(
    description = "swirl an image",
    cooldown = Duration::from_secs(3),
    access = Availability::Public,
    category = Category::Image,
    usage = "[image] <strength>",
    examples = ["https://link.to.my/image.png", "https://link.to.my/image.png 2.5", "https://link.to.my/image.png -1"],
    send_processing = true
)]
pub async fn swirl(ctxt: CommandCtxt<'_>, source: Image, strength: Option<f32>) -> anyhow::Result<()> {
    let result = ctxt
        .flux_handler()
        .swirl(
            source.0,
            strength,
            ctxt.data.author.id.get(),
            ctxt.data.guild_id.map(twilight_model::id::Id::get),
        )
        .await?;

    ctxt.reply(result).await?;

    Ok(())
}

#[command(
    description = "remove a caption from an image",
    cooldown = Duration::from_secs(3),
    access = Availability::Public,
    category = Category::Image,
    usage = "[image] <amount: percentage or lines>",
    examples = ["https://link.to.my/image.png", "https://link.to.my/image.png 50", "https://link.to.my/image.png 15%"],
    send_processing = true
)]
pub async fn uncaption(ctxt: CommandCtxt<'_>, source: Image, amount: Option<Word>) -> anyhow::Result<()> {
    let result = ctxt
        .flux_handler()
        .uncaption(
            source.0,
            amount.map(|a| a.0),
            ctxt.data.author.id.get(),
            ctxt.data.guild_id.map(twilight_model::id::Id::get),
        )
        .await?;

    ctxt.reply(result).await?;

    Ok(())
}

#[command(
    description = "put an image through a wormhole",
    cooldown = Duration::from_secs(3),
    access = Availability::Public,
    category = Category::Image,
    usage = "[image]",
    examples = ["https://link.to.my/image.png"],
    send_processing = true
)]
pub async fn wormhole(ctxt: CommandCtxt<'_>, source: Image) -> anyhow::Result<()> {
    let result = ctxt
        .flux_handler()
        .wormhole(source.0, ctxt.data.author.id.get(), ctxt.data.guild_id.map(twilight_model::id::Id::get))
        .await?;

    ctxt.reply(result).await?;

    Ok(())
}

#[command(
    description = "zoom into an image",
    cooldown = Duration::from_secs(3),
    access = Availability::Public,
    category = Category::Image,
    usage = "[image]",
    examples = ["https://link.to.my/image.png"],
    send_processing = true
)]
pub async fn zoom(ctxt: CommandCtxt<'_>, source: Image) -> anyhow::Result<()> {
    let result = ctxt
        .flux_handler()
        .zoom(source.0, ctxt.data.author.id.get(), ctxt.data.guild_id.map(twilight_model::id::Id::get))
        .await?;

    ctxt.reply(result).await?;

    Ok(())
}

#[command(
    description = "apply a motion blur effect to an image",
    aliases = ["zb"],
    cooldown = Duration::from_secs(3),
    access = Availability::Public,
    category = Category::Image,
    usage = "[image] <power: -20 to 20>",
    examples = ["https://link.to.my/image.png", "https://link.to.my/image.png -10.2", "https://link.to.my/image.png 5.4"],
    send_processing = true
)]
pub async fn zoomblur(ctxt: CommandCtxt<'_>, source: Image, power: Option<f32>) -> anyhow::Result<()> {
    let result = ctxt
        .flux_handler()
        .zoom_blur(
            source.0,
            power,
            ctxt.data.author.id.get(),
            ctxt.data.guild_id.map(twilight_model::id::Id::get),
        )
        .await?;

    ctxt.reply(result).await?;

    Ok(())
}
