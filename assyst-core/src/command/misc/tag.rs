use std::time::Duration;

use assyst_proc_macro::command;

use crate::command::{Availability, Category};
use crate::define_commandgroup;

use super::{CommandCtxt, Rest, Word};

#[command(description = "creates a tag", cooldown = Duration::from_secs(2), access = Availability::Public, category = Category::Misc, usage = "<name> <contents>")]
pub async fn create(ctxt: CommandCtxt<'_>, Word(name): Word, Rest(contents): Rest) -> anyhow::Result<()> {
    ctxt.reply(format!("create tag, name={name}, contents={contents}"))
        .await?;
    Ok(())
}

define_commandgroup! {
    name: tag,
    access: Availability::Public,
    category: Category::Misc,
    description: "tags",
    usage: "<create>",
    commands: [
        "create" => create
    ]
}
