use std::collections::HashMap;

use anyhow::{bail, Context};
use twilight_model::application::interaction::application_command::CommandOptionValue;
use twilight_util::builder::command::StringBuilder;

use super::arguments::ParseArgument;
use super::errors::TagParseError;
use super::Label;

macro_rules! flag_parse_argument {
    ($s:ty) => {
        impl ParseArgument for $s {
            fn as_command_option(name: &str) -> twilight_model::application::command::CommandOption {
                StringBuilder::new(name, "flags input").required(false).build()
            }

            async fn parse_raw_message(
                ctxt: &mut super::RawMessageParseCtxt<'_>,
                label: Label,
            ) -> Result<Self, super::errors::TagParseError> {
                let args = ctxt.rest_all(label);
                let parsed = Self::from_str(&args).map_err(|x| TagParseError::FlagParseError(x))?;
                Ok(parsed)
            }

            async fn parse_command_option(
                ctxt: &mut super::InteractionCommandParseCtxt<'_>,
            ) -> Result<Self, TagParseError> {
                let word = ctxt.next_option();

                if let Ok(option) = word {
                    if let CommandOptionValue::String(ref option) = option.value {
                        Ok(Self::from_str(&option[..]).map_err(|x| TagParseError::FlagParseError(x))?)
                    } else {
                        Err(TagParseError::MismatchedCommandOptionType((
                            "String (Flags)".to_owned(),
                            option.value.clone(),
                        )))
                    }
                } else {
                    Ok(Self::default())
                }
            }
        }
    };
}

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
            miri: raw_decode.get("miri").is_some(),
            asm: raw_decode.get("asm").is_some(),
            release: raw_decode.get("release").is_some(),
            clippy: raw_decode.get("clippy").is_some(),
            bench: raw_decode.get("bench").is_some(),
        };

        Ok(result)
    }
}
flag_parse_argument! { RustFlags }

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
            i_am_sure: raw_decode.get("i-am-sure").is_some(),
        };

        Ok(result)
    }
}
flag_parse_argument! { ColourRemoveAllFlags }

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
            .map(|x| x.clone().map(|y| y.parse::<u64>()))
            .flatten();

        let count = if let Some(inner) = count {
            Some(inner.context("Failed to parse translation count")?)
        } else {
            None
        };

        let result = Self {
            chain: raw_decode.get("chain").is_some(),
            count,
        };

        Ok(result)
    }
}
flag_parse_argument! { BadTranslateFlags }

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
            .map(|x| x.as_deref())
            .flatten()
            .map(|x| x.parse::<u64>())
            .unwrap_or(Ok(0))
            .context("Failed to parse optimisation level")?;

        let result = Self {
            verbose: raw_decode.get("verbose").is_some(),
            llir: raw_decode.get("llir").is_some(),
            opt,
            valgrind: raw_decode.get("valgrind").is_some(),
        };

        if result.llir && result.valgrind {
            bail!("Cannot set both valgrind and llir flags at the same time");
        }

        Ok(result)
    }
}
flag_parse_argument! { ChargeFlags }

#[derive(Default)]
pub struct DownloadFlags {
    pub audio: bool,
    pub quality: u64,
}
impl FlagDecode for DownloadFlags {
    fn from_str(input: &str) -> anyhow::Result<Self> {
        let mut valid_flags = HashMap::new();
        valid_flags.insert("quality", FlagType::WithValue);
        valid_flags.insert("audio", FlagType::NoValue);

        let raw_decode = flags_from_str(input, valid_flags)?;
        let result = Self {
            audio: raw_decode.get("audio").is_some(),
            quality: raw_decode
                .get("quality")
                .unwrap_or(&None)
                .clone()
                .unwrap_or("720".to_owned())
                .parse()
                .context("Provided quality is invalid")?,
        };

        Ok(result)
    }
}
flag_parse_argument! { DownloadFlags }

pub enum FlagType {
    WithValue,
    NoValue,
}

type ValidFlags = HashMap<&'static str, FlagType>;

pub trait FlagDecode {
    fn from_str(input: &str) -> anyhow::Result<Self>
    where
        Self: Sized;
}

pub fn flags_from_str(input: &str, valid_flags: ValidFlags) -> anyhow::Result<HashMap<String, Option<String>>> {
    let args = input.split_ascii_whitespace();
    let mut current_flag: Option<String> = None;
    let mut entries: HashMap<String, Option<String>> = HashMap::new();

    for arg in args {
        if arg.starts_with("--") && arg.len() > 2 {
            // prev flag present but no value, write to hashmap
            if let Some(ref c) = current_flag {
                let flag = valid_flags
                    .get(&c.as_ref())
                    .context(format!("Unrecognised flag: {c}"))?;

                if let FlagType::NoValue = flag {
                    entries.insert(c.clone(), None);
                    current_flag = Some(arg[2..].to_owned());
                } else {
                    bail!("Flag {c} expects a value, but none was provided");
                }
            } else {
                current_flag = Some(arg[2..].to_owned());
            }
        } else {
            // current flag present, this arg is its value
            if let Some(ref c) = current_flag {
                let flag = valid_flags
                    .get(&c.as_ref())
                    .context(format!("Unrecognised flag: {c}"))?;

                if let FlagType::WithValue = flag {
                    entries.insert(c.clone(), Some(arg.to_owned()));
                    current_flag = None;
                } else {
                    bail!("Flag {c} does not expect a value, even though one was provided");
                }
            }
        }
    }

    // handle case where we assign current flag in last arg, and return
    if let Some(c) = current_flag {
        let flag = valid_flags
            .get(&c.as_ref())
            .context(format!("Unrecognised flag: {c}"))?;
        if let FlagType::WithValue = flag {
            bail!("Flag {c} expects a value, but none was provided");
        } else {
            entries.insert(c.clone(), None);
        }
    }

    Ok(entries)
}
