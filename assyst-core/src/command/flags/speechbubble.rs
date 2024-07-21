use std::collections::HashMap;

use super::{flags_from_str, FlagDecode, FlagType, Label};

use crate::command::arguments::ParseArgument;
use crate::command::errors::TagParseError;
use crate::command::flags::StringBuilder;
use crate::command::{InteractionCommandParseCtxt, RawMessageParseCtxt};
use crate::flag_parse_argument;

use twilight_model::application::interaction::application_command::CommandOptionValue;

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
