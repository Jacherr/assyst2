use std::time::Duration;

use assyst_common::markdown::Markdown;
use assyst_proc_macro::command;
use rand::{thread_rng, Rng};

use crate::command::arguments::{Rest, Word};
use crate::command::{Availability, Category, CommandCtxt};
use crate::define_commandgroup;
use crate::rest::cooltext::STYLES;

#[command(
    description = "make some cool text",
    access = Availability::Public,
    cooldown = Duration::from_secs(2),
    category = Category::Services,
    examples = ["burning hello", "saint fancy", "random im random"],
    send_processing = true
)]
pub async fn default(ctxt: CommandCtxt<'_>, style: Word, text: Rest) -> anyhow::Result<()> {
    let style = if &style.0 == "random" {
        let rand = thread_rng().gen_range(0..STYLES.len());
        STYLES[rand].0
    } else {
        &style.0
    };

    let result = crate::rest::cooltext::cooltext(style, text.0.as_str()).await?;
    ctxt.reply((result, &format!("**Style:** `{style}`")[..])).await?;

    Ok(())
}

#[command(
    description = "list all cooltext options",
    access = Availability::Public,
    cooldown = Duration::from_secs(2),
    category = Category::Services,
    usage = "",
    examples = [""],
)]
pub async fn list(ctxt: CommandCtxt<'_>) -> anyhow::Result<()> {
    let options = STYLES.iter().map(|x| x.0.to_owned()).collect::<Vec<_>>();

    ctxt.reply(format!("**All Cooltext supported fonts:**\n{}", options.join(", ").codeblock(""))).await?;

    Ok(())
}

define_commandgroup! {
    name: cooltext,
    access: Availability::Public,
    category: Category::Services,
    aliases: ["ct", "funtext"],
    cooldown: Duration::from_secs(5),
    description: "Write some cool text",
    examples: ["random hello", "warp warpy text", "list"],
    usage: "[colour]",
    commands: [
        "list" => list
    ],
    default_interaction_subcommand: "run",
    default: default
}
