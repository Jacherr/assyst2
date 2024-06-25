use std::time::Duration;

use anyhow::{bail, Context};
use assyst_proc_macro::command;

use crate::command::arguments::{Rest, Word};
use crate::command::flags::BadTranslateFlags;
use crate::command::{Availability, Category, CommandCtxt};
use crate::rest::bad_translation::{
    bad_translate as bad_translate_default, bad_translate_with_count, translate_single, TranslateResult, Translation,
};

#[command(
    aliases = ["bt"],
    description = "Badly translate some text",
    access = Availability::Public,
    cooldown = Duration::from_secs(5),
    category = Category::Fun,
    usage = "[text]",
    examples = ["hello i love assyst"],
    flag_descriptions = [
        ("chain", "Show language chain"),
        ("count", "Set the amount of translations to perform")
    ],
    send_processing = true
)]
pub async fn bad_translate(ctxt: CommandCtxt<'_>, text: Rest, flags: BadTranslateFlags) -> anyhow::Result<()> {
    let TranslateResult {
        result: Translation { text, .. },
        translations,
    } = if let Some(count) = flags.count {
        if count < 10 {
            bad_translate_with_count(&ctxt.assyst().reqwest_client, &text.0, count as u32)
                .await
                .context("Failed to run bad translation")?
        } else {
            bail!("Translation count cannot exceed 10")
        }
    } else {
        bad_translate_default(&ctxt.assyst().reqwest_client, &text.0)
            .await
            .context("Failed to run bad translation")?
    };

    let mut output = format!("**Output:**\n{}", text);

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
