use std::time::Duration;

use anyhow::{bail, Context};
use assyst_common::markdown::Markdown;
use assyst_proc_macro::command;
use assyst_tag::ParseResult;

use crate::command::arguments::ParseArgument as _;
use crate::command::{Availability, Category};
use crate::define_commandgroup;

use super::{CommandCtxt, Rest, Word};

#[command(description = "creates a tag", cooldown = Duration::from_secs(2), access = Availability::Public, category = Category::Misc, usage = "<name> <contents>")]
pub async fn create(ctxt: CommandCtxt<'_>, Word(name): Word, Rest(contents): Rest) -> anyhow::Result<()> {
    ctxt.reply(format!("create tag, name={name}, contents={contents}"))
        .await?;
    Ok(())
}

#[command(description = "runs a tag", cooldown = Duration::from_secs(2), access = Availability::Public, category = Category::Misc, usage = "<args>")]
pub async fn default(ctxt: CommandCtxt<'_>, Word(tag_name): Word, arguments: Vec<Word>) -> anyhow::Result<()> {
    let Some(guild_id) = ctxt.data.guild_id else {
        bail!("tags can only be used in guilds")
    };

    let tag = ctxt
        .assyst()
        .database_handler
        .get_tag(guild_id as i64, &tag_name)
        .await?
        .context("Tag not found in this server")?;

    let (res, tag) = tokio::task::spawn_blocking(move || {
        let tag = tag;
        let arguments: Vec<&str> = arguments.iter().map(|Word(word)| &**word).collect();
        let tcx = TagContext;

        (assyst_tag::parse(&tag.data, &arguments, tcx), tag)
    })
    .await
    .expect("Tag task panicked");

    match res {
        Ok(ParseResult { output, attachment }) => ctxt.reply(output).await?,
        Err(err) => {
            ctxt.reply(assyst_tag::errors::format_error(&tag.data, err).codeblock("ansi"))
                .await?
        },
    }

    Ok(())
}

struct TagContext;

impl assyst_tag::Context for TagContext {
    fn execute_javascript(
        &self,
        code: &str,
        args: Vec<String>,
    ) -> anyhow::Result<assyst_common::eval::FakeEvalImageResponse> {
        todo!()
    }

    fn get_last_attachment(&self) -> anyhow::Result<String> {
        todo!()
    }

    fn get_avatar(&self, user_id: Option<u64>) -> anyhow::Result<String> {
        todo!()
    }

    fn download(&self, url: &str) -> anyhow::Result<String> {
        todo!()
    }

    fn channel_id(&self) -> anyhow::Result<u64> {
        todo!()
    }

    fn guild_id(&self) -> anyhow::Result<u64> {
        todo!()
    }

    fn user_id(&self) -> anyhow::Result<u64> {
        todo!()
    }

    fn user_tag(&self, id: Option<u64>) -> anyhow::Result<String> {
        todo!()
    }

    fn get_tag_contents(&self, tag: &str) -> anyhow::Result<String> {
        todo!()
    }
}

define_commandgroup! {
    name: tag,
    access: Availability::Public,
    category: Category::Misc,
    description: "tags",
    usage: "<create>",
    commands: [
        "create" => create
    ],
    default: default
}
