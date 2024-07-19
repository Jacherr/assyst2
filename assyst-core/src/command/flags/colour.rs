use std::collections::HashMap;

use super::{flags_from_str, FlagDecode, FlagType, Label};

use crate::command::arguments::ParseArgument;
use crate::command::errors::TagParseError;
use crate::command::flags::StringBuilder;
use crate::command::{InteractionCommandParseCtxt, RawMessageParseCtxt};
use crate::flag_parse_argument;

use twilight_model::application::interaction::application_command::CommandOptionValue;

#[derive(Default)]
pub struct ColourRemoveAllFlags {
    pub i_am_sure: bool,
}
impl FlagDecode for ColourRemoveAllFlags {
    fn from_str(input: &str) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let mut valid_flags = HashMap::new();
        valid_flags.insert("i-am-sure", FlagType::NoValue);

        let raw_decode = flags_from_str(input, valid_flags)?;
        let result = Self {
            i_am_sure: raw_decode.contains_key("i-am-sure"),
        };

        Ok(result)
    }
}
flag_parse_argument! { ColourRemoveAllFlags }
