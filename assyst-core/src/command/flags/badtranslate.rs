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
pub struct BadTranslateFlags {
    pub chain: bool,
    pub count: Option<u64>,
}
impl FlagDecode for BadTranslateFlags {
    fn from_str(input: &str) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let mut valid_flags = HashMap::new();
        valid_flags.insert("chain", FlagType::NoValue);
        valid_flags.insert("count", FlagType::WithValue);

        let raw_decode = flags_from_str(input, valid_flags)?;

        let count = raw_decode
            .get("count")
            .and_then(|x| x.clone().map(|y| y.parse::<u64>()));

        let count = if let Some(inner) = count {
            Some(inner.context("Failed to parse translation count")?)
        } else {
            None
        };

        let result = Self {
            chain: raw_decode.contains_key("chain"),
            count,
        };

        Ok(result)
    }
}
flag_parse_argument! { BadTranslateFlags }
