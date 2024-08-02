use std::collections::HashMap;
use std::time::Duration;

use assyst_proc_macro::command;
use twilight_model::application::interaction::application_command::CommandOptionValue;
use twilight_util::builder::command::StringBuilder;

use crate::command::arguments::{Image, ParseArgument, Rest};
use crate::command::errors::TagParseError;
use crate::command::flags::{flags_from_str, FlagDecode, FlagType};
use crate::command::{Availability, Category, CommandCtxt, InteractionCommandParseCtxt, Label, RawMessageParseCtxt};
use crate::flag_parse_argument;

#[command(
    description = "add a caption to an image",
    cooldown = Duration::from_secs(2),
    access = Availability::Public,
    category = Category::Image,
    usage = "[image] [caption] <...flags>",
    examples = ["https://link.to.my/image.png hello there", "https://link.to.my/image.png i am on the bottom --bottom", "https://link.to.my/image.png i am an inverted caption --black"],
    send_processing = true,
    flag_descriptions = [
        ("bottom", "Setting this flag puts the caption on the bottom of the image"),
        ("black", "Setting this flag inverts the caption"),
    ]
)]
pub async fn caption(ctxt: CommandCtxt<'_>, source: Image, text: Rest, flags: CaptionFlags) -> anyhow::Result<()> {
    let result = ctxt
        .flux_handler()
        .caption(
            source.0,
            text.0,
            flags.bottom,
            flags.black,
            ctxt.data.author.id.get(),
            ctxt.data.guild_id.map(|x| x.get()),
        )
        .await?;

    ctxt.reply(result).await?;

    Ok(())
}

#[derive(Default)]
pub struct CaptionFlags {
    pub bottom: bool,
    pub black: bool,
}
impl FlagDecode for CaptionFlags {
    fn from_str(input: &str) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let mut valid_flags = HashMap::new();
        valid_flags.insert("bottom", FlagType::NoValue);
        valid_flags.insert("black", FlagType::NoValue);

        let raw_decode = flags_from_str(input, valid_flags)?;

        let result = Self {
            bottom: raw_decode.contains_key("bottom"),
            black: raw_decode.contains_key("black"),
        };

        Ok(result)
    }
}
flag_parse_argument! { CaptionFlags }
