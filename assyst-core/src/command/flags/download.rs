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
pub struct DownloadFlags {
    pub audio: bool,
    pub quality: u64,
    pub verbose: bool,
}
impl FlagDecode for DownloadFlags {
    fn from_str(input: &str) -> anyhow::Result<Self> {
        let mut valid_flags = HashMap::new();
        valid_flags.insert("quality", FlagType::WithValue);
        valid_flags.insert("audio", FlagType::NoValue);
        valid_flags.insert("verbose", FlagType::NoValue);

        let raw_decode = flags_from_str(input, valid_flags)?;
        let result = Self {
            audio: raw_decode.contains_key("audio"),
            quality: raw_decode
                .get("quality")
                .unwrap_or(&None)
                .clone()
                .unwrap_or("720".to_owned())
                .parse()
                .context("Provided quality is invalid")?,
            verbose: raw_decode.contains_key("verbose"),
        };

        Ok(result)
    }
}
flag_parse_argument! { DownloadFlags }
