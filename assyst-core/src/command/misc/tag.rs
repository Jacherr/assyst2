use std::time::Duration;

use anyhow::{anyhow, bail, ensure, Context};
use assyst_common::markdown::Markdown;
use assyst_common::util::discord::{format_tag, get_avatar_url};
use assyst_common::util::string_from_likely_utf8;
use assyst_database::Tag;
use assyst_proc_macro::command;
use assyst_tag::ParseResult;
use tokio::runtime::Handle;
use twilight_model::channel::Message;
use twilight_model::id::Id;

use crate::assyst::ThreadSafeAssyst;
use crate::command::arguments::{Image, ImageUrl};
use crate::command::{Availability, Category};
use crate::define_commandgroup;
use crate::downloader::{download_content, ABSOLUTE_INPUT_FILE_SIZE_LIMIT_BYTES};
use crate::rest::eval::fake_eval;

use super::{CommandCtxt, Rest, Word};

#[command(description = "creates a tag", cooldown = Duration::from_secs(2), access = Availability::Public, category = Category::Misc, usage = "<name> <contents>")]
pub async fn create(ctxt: CommandCtxt<'_>, Word(name): Word, Rest(contents): Rest) -> anyhow::Result<()> {
    ctxt.reply(format!("create tag, name={name}, contents={contents}"))
        .await?;
    Ok(())
}

#[command(description = "runs a tag", cooldown = Duration::from_secs(2), access = Availability::Public, category = Category::Misc, usage = "<args>")]
pub async fn default(ctxt: CommandCtxt<'_>, Word(tag_name): Word, arguments: Vec<Word>) -> anyhow::Result<()> {
    let Some(guild_id) = ctxt.data.message.guild_id else {
        bail!("tags can only be used in guilds")
    };

    let tag = ctxt
        .assyst()
        .database_handler
        .get_tag(guild_id.get() as i64, &tag_name)
        .await?
        .context("Tag not found in this server")?;

    let assyst = ctxt.assyst().clone();
    let message = ctxt.data.message.clone();

    let (res, tag) = tokio::task::spawn_blocking(move || {
        let tag = tag;
        let arguments: Vec<&str> = arguments.iter().map(|Word(word)| &**word).collect();
        let tcx = TagContext {
            tokio: Handle::current(),
            message,
            assyst,
        };

        (assyst_tag::parse(&tag.data, &arguments, tcx), tag)
    })
    .await
    .expect("Tag task panicked");

    match res {
        Ok(ParseResult {
            output,
            attachment: None,
        }) => ctxt.reply(output).await?,
        Ok(ParseResult {
            output,
            attachment: Some((data, _)),
        }) => {
            ctxt.reply((Image(data), output.as_str())).await?;
        },
        Err(err) => {
            ctxt.reply(assyst_tag::errors::format_error(&tag.data, err).codeblock("ansi"))
                .await?
        },
    }

    Ok(())
}

struct TagContext {
    tokio: Handle,
    message: Message,
    assyst: ThreadSafeAssyst,
}

impl TagContext {
    fn guild_id(&self) -> u64 {
        self.message
            .guild_id
            .expect("tags can only be run in guilds; this invariant is ensured in the tag command")
            .get()
    }
}

impl assyst_tag::Context for TagContext {
    fn execute_javascript(
        &self,
        code: &str,
        args: Vec<String>,
    ) -> anyhow::Result<assyst_common::eval::FakeEvalImageResponse> {
        self.tokio
            .block_on(fake_eval(&self.assyst, code.into(), true, Some(&self.message), args))
    }

    fn get_last_attachment(&self) -> anyhow::Result<String> {
        let ImageUrl(attachment) = self
            .tokio
            .block_on(ImageUrl::from_channel_history(&self.assyst, self.message.channel_id))?;
        Ok(attachment)
    }

    fn get_avatar(&self, user_id: Option<u64>) -> anyhow::Result<String> {
        let user_id = user_id.unwrap_or(self.message.author.id.get());

        self.tokio.block_on(async {
            let user = self.assyst.http_client.user(Id::new(user_id)).await?;
            ensure!(user.status().get() != 404, "user not found");

            let user = user.model().await?;

            Ok(get_avatar_url(&user))
        })
    }

    fn download(&self, url: &str) -> anyhow::Result<String> {
        self.tokio
            .block_on(download_content(
                &self.assyst,
                url,
                ABSOLUTE_INPUT_FILE_SIZE_LIMIT_BYTES,
            ))
            .map(string_from_likely_utf8)
            .map_err(Into::into)
    }

    fn channel_id(&self) -> anyhow::Result<u64> {
        Ok(self.message.channel_id.get())
    }

    fn guild_id(&self) -> anyhow::Result<u64> {
        Ok(TagContext::guild_id(self))
    }

    fn user_id(&self) -> anyhow::Result<u64> {
        Ok(self.message.author.id.get())
    }

    fn user_tag(&self, id: Option<u64>) -> anyhow::Result<String> {
        if let Some(id) = id {
            self.tokio.block_on(async {
                let user = self.assyst.http_client.user(Id::new(id)).await?;
                ensure!(user.status().get() != 404, "user not found");

                Ok(format_tag(&user.model().await?))
            })
        } else {
            Ok(format_tag(&self.message.author))
        }
    }

    fn get_tag_contents(&self, tag: &str) -> anyhow::Result<String> {
        let tag = self
            .tokio
            .block_on(async { self.assyst.database_handler.get_tag(self.guild_id() as i64, tag).await });

        match tag {
            Ok(Some(Tag { data, .. })) => Ok(data),
            Ok(None) => Err(anyhow!("Tag not found")),
            Err(e) => Err(e.into()),
        }
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
