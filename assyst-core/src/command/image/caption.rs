use std::collections::HashMap;
use std::time::Duration;

use assyst_proc_macro::command;
use twilight_util::builder::command::BooleanBuilder;

use crate::command::arguments::{Image, ParseArgument, Rest};
use crate::command::errors::TagParseError;
use crate::command::flags::{flags_from_str, FlagDecode, FlagType};
use crate::command::{Availability, Category, CommandCtxt};
use crate::int_arg_bool;

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
            ctxt.data.guild_id.map(twilight_model::id::Id::get),
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
impl ParseArgument for CaptionFlags {
    fn as_command_options(_: &str) -> Vec<twilight_model::application::command::CommandOption> {
        vec![
            BooleanBuilder::new("bottom", "put the caption on the bottom")
                .required(false)
                .build(),
            BooleanBuilder::new("black", "invert the caption")
                .required(false)
                .build(),
        ]
    }

    async fn parse_raw_message(
        ctxt: &mut crate::command::RawMessageParseCtxt<'_>,
        label: crate::command::Label,
    ) -> Result<Self, crate::command::errors::TagParseError> {
        let args = ctxt.rest_all(label);
        let parsed = Self::from_str(&args).map_err(TagParseError::FlagParseError)?;
        Ok(parsed)
    }

    async fn parse_command_option(
        ctxt: &mut crate::command::InteractionCommandParseCtxt<'_>,
        _: crate::command::Label,
    ) -> Result<Self, TagParseError> {
        let bottom = int_arg_bool!(ctxt, "bottom", false);
        let black = int_arg_bool!(ctxt, "black", false);

        Ok(Self { bottom, black })
    }
}
