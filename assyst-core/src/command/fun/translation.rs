use std::collections::HashMap;
use std::time::Duration;

use anyhow::{bail, Context};
use assyst_common::util::{normalize_emojis, normalize_mentions, table};
use assyst_proc_macro::command;
use assyst_string_fmt::Markdown;
use twilight_util::builder::command::{BooleanBuilder, IntegerBuilder};

use crate::command::arguments::{ParseArgument, Rest, Word};
use crate::command::errors::TagParseError;
use crate::command::flags::{flags_from_str, FlagDecode, FlagType};
use crate::command::{Availability, Category, CommandCtxt};
use crate::rest::bad_translation::{
    bad_translate as bad_translate_default, bad_translate_with_count, get_languages, translate_single, TranslateResult,
    Translation,
};
use crate::{int_arg_bool, int_arg_u64_opt};

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
impl ParseArgument for BadTranslateFlags {
    fn as_command_options(_: &str) -> Vec<twilight_model::application::command::CommandOption> {
        vec![
            IntegerBuilder::new("count", "amount of translations")
                .required(false)
                .min_value(1)
                .max_value(10)
                .build(),
            BooleanBuilder::new("chain", "show language chain")
                .required(false)
                .build(),
        ]
    }

    async fn parse_raw_message(
        ctxt: &mut crate::command::RawMessageParseCtxt<'_>,
        label: crate::command::Label,
    ) -> Result<Self, crate::command::errors::TagParseError> {
        let args = ctxt.rest_all(label);
        let parsed = Self::from_str(&args).map_err(TagParseError::FlagParseError)?;
        Ok(parsed)
    }

    async fn parse_command_option(
        ctxt: &mut crate::command::InteractionCommandParseCtxt<'_>,
        _: crate::command::Label,
    ) -> Result<Self, TagParseError> {
        let count = int_arg_u64_opt!(ctxt, "count");
        let chain = int_arg_bool!(ctxt, "chain", false);

        Ok(Self { count, chain })
    }
}

#[command(
    name = "badtranslate",
    aliases = ["bt"],
    description = "Badly translate some text",
    access = Availability::Public,
    cooldown = Duration::from_secs(5),
    category = Category::Fun,
    usage = "[text|\"languages\"]",
    examples = ["hello i love assyst", "languages"],
    flag_descriptions = [
        ("chain", "Show language chain"),
        ("count", "Set the amount of translations to perform")
    ],
    send_processing = true,
    context_menu_command = "Bad Translate"
)]
pub async fn bad_translate(ctxt: CommandCtxt<'_>, text: Rest, flags: BadTranslateFlags) -> anyhow::Result<()> {
    if text.0 == "languages" {
        let languages = get_languages(&ctxt.assyst().reqwest_client)
            .await
            .context("Failed to fetch translation languages")?;

        let formatted = table::generate_list("Code", "Name", &languages);

        ctxt.reply(formatted.codeblock("")).await?;

        return Ok(());
    };

    let text = normalize_emojis(&text.0);
    let text = normalize_mentions(
        &text,
        &ctxt.data.message.map(|x| &x.mentions).cloned().unwrap_or_default(),
    );

    let TranslateResult {
        result: Translation { text, .. },
        translations,
    } = if let Some(count) = flags.count {
        if count < 10 {
            bad_translate_with_count(&ctxt.assyst().reqwest_client, &text, count as u32)
                .await
                .context("Failed to run bad translation")?
        } else {
            bail!("Translation count cannot exceed 10")
        }
    } else {
        bad_translate_default(&ctxt.assyst().reqwest_client, &text)
            .await
            .context("Failed to run bad translation")?
    };

    let mut output = format!("**Output:**\n{text}");

    if flags.chain {
        output += "\n\n**Language chain:**\n";

        for (idx, translation) in translations.iter().enumerate() {
            output += &format!("{}) {}: {}\n", idx + 1, translation.lang, translation.text);
        }
    }

    ctxt.reply(output).await?;

    Ok(())
}

#[command(
    aliases = ["tr"],
    description = "Translate some text",
    access = Availability::Public,
    cooldown = Duration::from_secs(5),
    category = Category::Fun,
    usage = "[language] [text]",
    examples = ["en kurwa"],
)]
pub async fn translate(ctxt: CommandCtxt<'_>, language: Word, text: Rest) -> anyhow::Result<()> {
    let TranslateResult {
        result: Translation { text, .. },
        ..
    } = translate_single(&ctxt.assyst().reqwest_client, &text.0, &language.0)
        .await
        .context("Failed to translate text")?;

    ctxt.reply(text).await?;

    Ok(())
}
