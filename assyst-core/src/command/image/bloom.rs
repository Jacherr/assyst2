use std::collections::HashMap;
use std::time::Duration;

use anyhow::Context;
use assyst_proc_macro::command;
use twilight_util::builder::command::IntegerBuilder;

use crate::command::arguments::{Image, ParseArgument};
use crate::command::errors::TagParseError;
use crate::command::flags::{flags_from_str, FlagDecode, FlagType};
use crate::command::{Availability, Category, CommandCtxt};
use crate::int_arg_u64_opt;

#[derive(Default)]
pub struct BloomFlags {
    pub radius: Option<u64>,
    pub brightness: Option<u64>,
    pub sharpness: Option<u64>,
}
impl FlagDecode for BloomFlags {
    fn from_str(input: &str) -> anyhow::Result<Self> {
        let mut valid_flags = HashMap::new();
        valid_flags.insert("radius", FlagType::WithValue);
        valid_flags.insert("sharpness", FlagType::WithValue);
        valid_flags.insert("brightness", FlagType::WithValue);

        let raw_decode = flags_from_str(input, valid_flags)?;
        let result = Self {
            radius: raw_decode
                .get("radius")
                .unwrap_or(&None)
                .clone()
                .map(|x| x.parse().context("Provided radius is invalid"))
                .transpose()?,
            sharpness: raw_decode
                .get("sharpness")
                .unwrap_or(&None)
                .clone()
                .map(|x| x.parse().context("Provided sharpness is invalid"))
                .transpose()?,
            brightness: raw_decode
                .get("brightness")
                .unwrap_or(&None)
                .clone()
                .map(|x| x.parse().context("Provided brightness is invalid"))
                .transpose()?,
        };

        Ok(result)
    }
}
impl ParseArgument for BloomFlags {
    fn as_command_options(_: &str) -> Vec<twilight_model::application::command::CommandOption> {
        vec![
            IntegerBuilder::new("radius", "bloom radius").required(false).build(),
            IntegerBuilder::new("sharpness", "bloom sharpness")
                .required(false)
                .build(),
            IntegerBuilder::new("brightness", "bloom brightness")
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
        let radius = int_arg_u64_opt!(ctxt, "radius");
        let sharpness = int_arg_u64_opt!(ctxt, "sharpness");
        let brightness = int_arg_u64_opt!(ctxt, "brightness");

        Ok(Self {
            radius,
            sharpness,
            brightness,
        })
    }
}

#[command(
    description = "add bloom to an image",
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
            ctxt.data.guild_id.map(|x| x.get()),
        )
        .await?;

    ctxt.reply(result).await?;

    Ok(())
}
