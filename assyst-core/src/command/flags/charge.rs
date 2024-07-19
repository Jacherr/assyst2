use std::collections::HashMap;

use super::{flags_from_str, FlagDecode, FlagType, Label};

use crate::command::arguments::ParseArgument;
use crate::command::errors::TagParseError;
use crate::command::flags::StringBuilder;
use crate::command::{InteractionCommandParseCtxt, RawMessageParseCtxt};
use crate::flag_parse_argument;

use anyhow::{bail, Context};
use twilight_model::application::interaction::application_command::CommandOptionValue;

#[derive(Default)]
pub struct ChargeFlags {
    pub verbose: bool,
    pub llir: bool,
    pub opt: u64,
    pub valgrind: bool,
}
impl FlagDecode for ChargeFlags {
    fn from_str(input: &str) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let mut valid_flags = HashMap::new();
        valid_flags.insert("verbose", FlagType::NoValue);
        valid_flags.insert("llir", FlagType::NoValue);
        valid_flags.insert("opt", FlagType::WithValue);
        valid_flags.insert("valgrind", FlagType::NoValue);

        let raw_decode = flags_from_str(input, valid_flags)?;
        let opt = raw_decode
            .get("opt")
            .and_then(|x| x.as_deref())
            .map(|x| x.parse::<u64>())
            .unwrap_or(Ok(0))
            .context("Failed to parse optimisation level")?;

        let result = Self {
            verbose: raw_decode.contains_key("verbose"),
            llir: raw_decode.contains_key("llir"),
            opt,
            valgrind: raw_decode.contains_key("valgrind"),
        };

        if result.llir && result.valgrind {
            bail!("Cannot set both valgrind and llir flags at the same time");
        }

        Ok(result)
    }
}
flag_parse_argument! { ChargeFlags }
