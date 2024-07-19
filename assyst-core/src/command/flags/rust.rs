use std::collections::HashMap;

use super::{flags_from_str, FlagDecode, FlagType, Label};

use crate::command::arguments::ParseArgument;
use crate::command::errors::TagParseError;
use crate::command::flags::StringBuilder;
use crate::command::{InteractionCommandParseCtxt, RawMessageParseCtxt};
use crate::flag_parse_argument;

use twilight_model::application::interaction::application_command::CommandOptionValue;

#[derive(Default)]
pub struct RustFlags {
    pub miri: bool,
    pub asm: bool,
    pub clippy: bool,
    pub bench: bool,
    pub release: bool,
}
impl FlagDecode for RustFlags {
    fn from_str(input: &str) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let mut valid_flags = HashMap::new();
        valid_flags.insert("miri", FlagType::NoValue);
        valid_flags.insert("release", FlagType::NoValue);
        valid_flags.insert("asm", FlagType::NoValue);
        valid_flags.insert("clippy", FlagType::NoValue);
        valid_flags.insert("bench", FlagType::NoValue);

        let raw_decode = flags_from_str(input, valid_flags)?;
        let result = Self {
            miri: raw_decode.contains_key("miri"),
            asm: raw_decode.contains_key("asm"),
            release: raw_decode.contains_key("release"),
            clippy: raw_decode.contains_key("clippy"),
            bench: raw_decode.contains_key("bench"),
        };

        Ok(result)
    }
}
flag_parse_argument! { RustFlags }
