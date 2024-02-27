use std::time::Duration;

use assyst_proc_macro::command;

use super::arguments::{Image, Rest};
use crate::command::{Availability, Category, CommandCtxt};

#[command(
    description = "add a caption to an image",
    cooldown = Duration::from_secs(1),
    access = Availability::Public,
    category = Category::Misc,
    usage = "[image] [caption]",
    examples = ["https://link.to.my/image.png"],
    send_processing = true
)]
pub async fn caption(ctxt: CommandCtxt<'_>, source: Image, text: Rest) -> anyhow::Result<()> {
    let result = ctxt
        .assyst()
        .wsi_handler
        .caption(source.0, text.0, ctxt.data.author.id.get())
        .await?;

    ctxt.reply(result).await?;

    Ok(())
}
