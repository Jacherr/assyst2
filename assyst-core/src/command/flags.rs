use std::collections::HashMap;

use anyhow::{bail, Context};

#[macro_export]
macro_rules! flag_parse_argument {
    ($s:ty) => {
        impl $crate::command::arguments::ParseArgument for $s {
            fn as_command_option(name: &str) -> twilight_model::application::command::CommandOption {
                twilight_util::builder::command::StringBuilder::new(name, "flags input")
                    .required(false)
                    .build()
            }

            async fn parse_raw_message(
                ctxt: &mut $crate::command::RawMessageParseCtxt<'_>,
                label: $crate::command::Label,
            ) -> Result<Self, $crate::command::TagParseError> {
                let args = ctxt.rest_all(label);
                let parsed = Self::from_str(&args).map_err(|x| $crate::command::TagParseError::FlagParseError(x))?;
                Ok(parsed)
            }

            async fn parse_command_option(
                ctxt: &mut $crate::command::InteractionCommandParseCtxt<'_>,
                label: $crate::command::Label,
            ) -> Result<Self, $crate::command::TagParseError> {
                let word = &ctxt.option_by_name(&label.unwrap().0);

                if let Ok(option) = word {
                    if let twilight_model::application::interaction::application_command::CommandOptionValue::String(
                        ref option,
                    ) = option.value
                    {
                        Ok(Self::from_str(&option[..])
                            .map_err(|x| $crate::command::TagParseError::FlagParseError(x))?)
                    } else {
                        Err($crate::command::TagParseError::MismatchedCommandOptionType((
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
        if (arg.starts_with("--") && arg.len() > 2) || (arg.starts_with("—") && arg.len() > 1) {
            let arglen = if arg.starts_with("--") { 2 } else { 1 };

            // prev flag present but no value, write to hashmap
            if let Some(ref c) = current_flag {
                let flag = valid_flags
                    .get(&c.as_ref())
                    .context(format!("Unrecognised flag: {c}"))?;

                if let FlagType::NoValue = flag {
                    entries.insert(c.clone(), None);
                    current_flag = Some(arg.chars().skip(arglen).collect::<String>());
                } else {
                    bail!("Flag {c} expects a value, but none was provided");
                }
            } else {
                current_flag = Some(arg.chars().skip(arglen).collect::<String>());
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
