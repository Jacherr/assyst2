use std::collections::HashMap;

use super::{flags_from_str, FlagDecode, FlagType, Label};

use crate::command::arguments::ParseArgument;
use crate::command::errors::TagParseError;
use crate::command::flags::StringBuilder;
use crate::command::{InteractionCommandParseCtxt, RawMessageParseCtxt};
use crate::flag_parse_argument;

use anyhow::Context;
use twilight_model::application::interaction::application_command::CommandOptionValue;

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
flag_parse_argument! { BloomFlags }
