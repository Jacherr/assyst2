use std::time::Duration;

use assyst_proc_macro::command;

use super::arguments::Rest;
use super::CommandCtxt;

use crate::command::{Availability, Category};
use crate::rest::r34::get_random_r34;

#[command(
    name = "rule34",
    aliases = ["r34"],
    description = "get random image from r34",
    access = Availability::Public,
    cooldown = Duration::from_secs(2),
    category = Category::Services,
    usage = "<tags separated by spaces>",
    examples = ["", "assyst"],
    age_restricted = true
)]
pub async fn r34(ctxt: CommandCtxt<'_>, tags: Rest) -> anyhow::Result<()> {
    let result = get_random_r34(ctxt.assyst().clone(), &tags.0).await?;
    let reply = format!("{} (Score: {})", result.file_url, result.score);

    ctxt.reply(reply).await?;

    Ok(())
}
