use std::collections::HashMap;

use super::{flags_from_str, FlagDecode, FlagType, Label};

use crate::command::arguments::ParseArgument;
use crate::command::errors::TagParseError;
use crate::command::flags::StringBuilder;
use crate::command::{InteractionCommandParseCtxt, RawMessageParseCtxt};
use crate::flag_parse_argument;

use twilight_model::application::interaction::application_command::CommandOptionValue;

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
