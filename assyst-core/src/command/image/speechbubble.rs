use std::collections::HashMap;
use std::time::Duration;

use assyst_proc_macro::command;
use twilight_model::application::interaction::application_command::CommandOptionValue;
use twilight_util::builder::command::StringBuilder;

use crate::command::arguments::{Image, ParseArgument};
use crate::command::errors::TagParseError;
use crate::command::flags::{flags_from_str, FlagDecode, FlagType};
use crate::command::{Availability, Category, CommandCtxt, InteractionCommandParseCtxt, Label, RawMessageParseCtxt};
use crate::flag_parse_argument;

#[command(
    description = "add a speechbubble to an image",
    aliases = ["speech"],
    cooldown = Duration::from_secs(2),
    access = Availability::Public,
    category = Category::Image,
    usage = "[image] <...flags>",
    examples = ["https://link.to.my/image.png", "https://link.to.my/image.png --solid"],
    send_processing = true,
    flag_descriptions = [
        ("solid", "Setting this flag will make the speech bubble a solid white instead of transparent"),
    ]
)]
pub async fn speechbubble(ctxt: CommandCtxt<'_>, source: Image, flags: SpeechBubbleFlags) -> anyhow::Result<()> {
    let result = ctxt
        .flux_handler()
        .speech_bubble(source.0, flags.solid, ctxt.data.author.id.get())
        .await?;

    ctxt.reply(result).await?;

    Ok(())
}

#[derive(Default)]
pub struct SpeechBubbleFlags {
    pub solid: bool,
}
impl FlagDecode for SpeechBubbleFlags {
    fn from_str(input: &str) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let mut valid_flags = HashMap::new();
        valid_flags.insert("solid", FlagType::NoValue);

        let raw_decode = flags_from_str(input, valid_flags)?;

        let result = Self {
            solid: raw_decode.contains_key("solid"),
        };

        Ok(result)
    }
}
flag_parse_argument! { SpeechBubbleFlags }
