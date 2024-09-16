use std::collections::HashMap;

use anyhow::{bail, Context};

#[macro_export]
macro_rules! int_arg_u64 {
    ($ctxt:expr, $s:expr, $d:expr) => {{
        let inner = $ctxt
            .option_by_name($s)
            .map(|o| o.value.clone())
            .unwrap_or(twilight_model::application::interaction::application_command::CommandOptionValue::Integer($d));

        let inner =
            if let twilight_model::application::interaction::application_command::CommandOptionValue::Integer(option) =
                inner
            {
                option as u64
            } else {
                panic!("download {} wrong arg type", $s);
            };

        inner
    }};
}

#[macro_export]
macro_rules! int_arg_u64_opt {
    ($ctxt:expr, $s:expr) => {{
        let inner = $ctxt.option_by_name($s).map(|o| o.value.clone());

        let inner = if let Ok(
            twilight_model::application::interaction::application_command::CommandOptionValue::Integer(option),
        ) = inner
        {
            Some(option as u64)
        } else {
            None
        };

        inner
    }};
}

#[macro_export]
macro_rules! int_arg_bool {
    ($ctxt:expr, $s:expr, $d:expr) => {{
        let inner = $ctxt
            .option_by_name($s)
            .map(|o| o.value.clone())
            .unwrap_or(twilight_model::application::interaction::application_command::CommandOptionValue::Boolean($d));

        let inner =
            if let twilight_model::application::interaction::application_command::CommandOptionValue::Boolean(option) =
                inner
            {
                option
            } else {
                panic!("download {} wrong arg type", $s);
            };

        inner
    }};
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
