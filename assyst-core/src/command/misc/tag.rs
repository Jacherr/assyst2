use std::fmt::Write;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, bail, ensure, Context};
use assyst_common::markdown::Markdown;
use assyst_common::util::discord::{format_discord_timestamp, format_tag, get_avatar_url};
use assyst_common::util::string_from_likely_utf8;
use assyst_database::model::tag::Tag;
use assyst_proc_macro::command;
use assyst_tag::parser::ParseMode;
use assyst_tag::ParseResult;
use tokio::runtime::Handle;
use twilight_model::channel::Message;
use twilight_model::id::marker::ChannelMarker;
use twilight_model::id::Id;

use super::CommandCtxt;
use crate::assyst::ThreadSafeAssyst;
use crate::command::arguments::{Image, ImageUrl, RestNoFlags, User, Word};
use crate::command::{Availability, Category};
use crate::define_commandgroup;
use crate::downloader::{download_content, ABSOLUTE_INPUT_FILE_SIZE_LIMIT_BYTES};
use crate::rest::eval::fake_eval;

const DEFAULT_LIST_COUNT: i64 = 15;

#[command(
    description = "create a tag",
    aliases = ["add"],
    cooldown = Duration::from_secs(2),
    access = Availability::Public,
    category = Category::Misc,
    usage = "[name] [contents]",
    examples = ["test hello", "script 1+2 is: {js:1+2}"]
)]
pub async fn create(ctxt: CommandCtxt<'_>, name: Word, contents: RestNoFlags) -> anyhow::Result<()> {
    const RESERVED_NAMES: &[&str] = &["create", "add", "edit", "raw", "remove", "delete", "list", "info"];

    let author = ctxt.data.author.id.get();
    let Some(guild_id) = ctxt.data.guild_id else {
        bail!("Tags can only be created in guilds.")
    };
    ensure!(name.0.len() < 20, "Tag names cannot exceed 20 characters.");
    ensure!(
        !RESERVED_NAMES.contains(&&name.0[..]),
        "Tag name cannot be a reserved word."
    );

    let tag = Tag {
        name: name.0.to_ascii_lowercase(),
        guild_id: guild_id.get() as i64,
        data: contents.0,
        author: author as i64,
        created_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as i64,
    };

    let success = tag
        .set(&ctxt.assyst().database_handler)
        .await
        .context("Failed to create tag")?;

    ensure!(success, "That tag name is already used in this server.");

    ctxt.reply(format!(
        "Successfully created tag {}",
        tag.name.to_ascii_lowercase().codestring()
    ))
    .await?;

    Ok(())
}

#[command(
    description = "edit a tag that you own",
    cooldown = Duration::from_secs(2),
    access = Availability::Public,
    category = Category::Misc,
    usage = "[name] [contents]",
    examples = ["test hello there", "script 2+2 is: {js:2+2}"]
)]
pub async fn edit(ctxt: CommandCtxt<'_>, name: Word, contents: RestNoFlags) -> anyhow::Result<()> {
    let author = ctxt.data.author.id.get();
    let Some(guild_id) = ctxt.data.guild_id else {
        bail!("Tags can only be edited in guilds.")
    };

    let success = Tag::edit(
        &ctxt.assyst().database_handler,
        author as i64,
        guild_id.get() as i64,
        &name.0.to_ascii_lowercase(),
        &contents.0,
    )
    .await
    .context("Failed to edit tag")?;

    ensure!(success, "Failed to edit that tag. Does it exist, and do you own it?");

    ctxt.reply(format!(
        "Successfully edited tag {}",
        contents.0.to_ascii_lowercase().codestring()
    ))
    .await?;

    Ok(())
}

#[command(
    description = "delete a tag that you own (server managers can delete any tag in the server)",
    aliases = ["remove"],
    cooldown = Duration::from_secs(2),
    access = Availability::Public,
    category = Category::Misc,
    usage = "[name]",
    examples = ["test", "script"]
)]
pub async fn delete(ctxt: CommandCtxt<'_>, name: Word) -> anyhow::Result<()> {
    let author = ctxt.data.author.id.get();
    let Some(guild_id) = ctxt.data.guild_id else {
        bail!("Tags can only be deleted in guilds.")
    };

    let success = if ctxt
        .assyst()
        .rest_cache_handler
        .user_is_guild_manager(guild_id.get(), author)
        .await
        .context("Failed to fetch user permissions")?
    {
        Tag::delete_force(
            &ctxt.assyst().database_handler,
            &name.0.to_ascii_lowercase(),
            guild_id.get() as i64,
        )
        .await
        .context("Failed to delete tag")?
    } else {
        Tag::delete(
            &ctxt.assyst().database_handler,
            &name.0.to_ascii_lowercase(),
            guild_id.get() as i64,
            author as i64,
        )
        .await
        .context("Failed to delete tag")?
    };

    ensure!(success, "Failed to delete that tag. Does it exist, and do you own it?");

    ctxt.reply(format!(
        "Successfully edited tag {}",
        name.0.to_ascii_lowercase().codestring()
    ))
    .await?;

    Ok(())
}

#[command(
    description = "list tags in the server (or owned by a certain user in the server)",
    cooldown = Duration::from_secs(2),
    access = Availability::Public,
    category = Category::Misc,
    usage = "<page> <user id|mention>",
    examples = ["1 @jacher", "1"]
)]
pub async fn list(ctxt: CommandCtxt<'_>, page: u64, user: Option<User>) -> anyhow::Result<()> {
    let Some(guild_id) = ctxt.data.guild_id else {
        bail!("Tags can only be listed in guilds.")
    };

    // user-specific search if arg is a mention
    let user_id: Option<i64> = user.map(|x| x.0.id.get() as i64);

    ensure!(page >= 1, "Page must be greater or equal to 1");

    let offset = (page as i64 - 1) * DEFAULT_LIST_COUNT;
    let count = match user_id {
        Some(u) => Tag::get_count_for_user(&ctxt.assyst().database_handler, guild_id.get() as i64, u)
            .await
            .context("Failed to get tag count for user in guild")?,
        None => Tag::get_count_in_guild(&ctxt.assyst().database_handler, guild_id.get() as i64)
            .await
            .context("Failed to get tag count in guild")?,
    };

    ensure!(count > 0, "No tags found for the requested filter");
    let pages = (count as f64 / DEFAULT_LIST_COUNT as f64).ceil() as i64;
    ensure!(pages >= page as i64, "Cannot go beyond final page");

    let tags = match user_id {
        Some(u) => {
            Tag::get_paged_for_user(
                &ctxt.assyst().database_handler,
                guild_id.get() as i64,
                u,
                offset,
                DEFAULT_LIST_COUNT,
            )
            .await?
        },
        None => {
            Tag::get_paged(
                &ctxt.assyst().database_handler,
                guild_id.get() as i64,
                offset,
                DEFAULT_LIST_COUNT,
            )
            .await?
        },
    };

    let mut message = format!(
        "🗒️ **Tags in this server{0}**\nView a tag by running `{1}t <name>`, or go to the next page by running `{1}t list {2}`\n\n",
        {
            match user_id {
                Some(u) => format!(" for user <@{u}>"),
                None => "".to_owned(),
            }
        },
        ctxt.data.calling_prefix,
        page + 1
    );

    for (index, tag) in tags.iter().enumerate() {
        let offset = (index as i64) + offset + 1;
        writeln!(
            message,
            "{}. {} {}",
            offset,
            tag.name.to_ascii_lowercase(),
            match user_id {
                Some(_) => "".to_owned(),
                None => format!("(<@{}>)", tag.author),
            }
        )?;
    }

    write!(
        message,
        "\nShowing {} tags (page {page}/{pages}) ({count} total tags)",
        tags.len()
    )?;

    ctxt.reply(message).await?;

    Ok(())
}

#[command(
    description = "get information about a tag in the server",
    cooldown = Duration::from_secs(2),
    access = Availability::Public,
    category = Category::Misc,
    usage = "[name]",
    examples = ["test", "script"]
)]
pub async fn info(ctxt: CommandCtxt<'_>, name: Word) -> anyhow::Result<()> {
    let Some(guild_id) = ctxt.data.guild_id else {
        bail!("Tag information can only be fetched in guilds.")
    };

    let tag = Tag::get(
        &ctxt.assyst().database_handler,
        guild_id.get() as i64,
        &name.0.to_ascii_lowercase(),
    )
    .await?
    .context("Tag not found in this server.")?;

    let fmt = format_discord_timestamp(tag.created_at as u64);
    let message = format!(
        "🗒️ **Tag information: **{}\n\nAuthor: <@{}>\nCreated: {}",
        tag.name.to_ascii_lowercase(),
        tag.author,
        fmt
    );

    ctxt.reply(message).await?;

    Ok(())
}

#[command(
    description = "get the raw content of a tag without parsing it",
    cooldown = Duration::from_secs(2),
    access = Availability::Public,
    category = Category::Misc,
    usage = "[name]",
    examples = ["test", "script"]
)]
pub async fn raw(ctxt: CommandCtxt<'_>, name: Word) -> anyhow::Result<()> {
    let Some(guild_id) = ctxt.data.guild_id else {
        bail!("Tag raw content can only be fetched in guilds.")
    };

    let tag = Tag::get(
        &ctxt.assyst().database_handler,
        guild_id.get() as i64,
        &name.0.to_ascii_lowercase(),
    )
    .await?
    .context("Tag not found in this server.")?;

    ctxt.reply(tag.data.codeblock("")).await?;

    Ok(())
}

#[command(
    description = "search for tags in a server based on a query",
    cooldown = Duration::from_secs(2),
    access = Availability::Public,
    category = Category::Misc,
    usage = "[query] <page> <user id|mention>",
    examples = ["1 test @jacher", "1 test"]
)]
pub async fn search(ctxt: CommandCtxt<'_>, page: u64, query: Word, user: Option<User>) -> anyhow::Result<()> {
    let Some(guild_id) = ctxt.data.guild_id else {
        bail!("Tags can only be listed in guilds.")
    };

    // user-specific search if arg is a mention
    let user_id: Option<i64> = user.map(|x| x.0.id.get() as i64);

    ensure!(page >= 1, "Page must be greater or equal to 1");

    let offset = (page as i64 - 1) * DEFAULT_LIST_COUNT;
    let tags = match user_id {
        Some(u) => {
            Tag::search_in_guild_for_user(
                &ctxt.assyst().database_handler,
                guild_id.get() as i64,
                u,
                &query.0.to_ascii_lowercase(),
            )
            .await?
        },
        None => {
            Tag::search_in_guild(
                &ctxt.assyst().database_handler,
                guild_id.get() as i64,
                &query.0.to_ascii_lowercase(),
            )
            .await?
        },
    };
    let count = tags.len();

    ensure!(count > 0, "No tags found for the requested filter");
    let pages = (count as f64 / DEFAULT_LIST_COUNT as f64).ceil() as i64;
    ensure!(pages >= page as i64, "Cannot go beyond final page");

    let tags = &tags[offset as usize..(offset + DEFAULT_LIST_COUNT) as usize];

    let mut message = format!(
        "🗒️ **Tags in this server matching query {0}{1}**\nView a tag by running `{2}t <name>`, or go to the next page by running `{2}t list {3} {0}`\n\n",
        query.0,
        {
            match user_id {
                Some(u) => format!(" for user <@{u}>"),
                None => "".to_owned(),
            }
        },
        ctxt.data.calling_prefix,
        page + 1
    );

    for (index, tag) in tags.iter().enumerate() {
        let offset = (index as i64) + offset + 1;
        writeln!(
            message,
            "{}. {} {}",
            offset,
            tag.name.to_ascii_lowercase(),
            match user_id {
                Some(_) => "".to_owned(),
                None => format!("(<@{}>)", tag.author),
            }
        )?;
    }

    write!(
        message,
        "\nShowing {} tags (page {page}/{pages}) ({count} total tags)",
        tags.len()
    )?;

    ctxt.reply(message).await?;

    Ok(())
}

#[command(
    description = "run a tag in the current server",
    cooldown = Duration::from_secs(2),
    access = Availability::Public,
    category = Category::Misc,
    usage = "[tag name] <arguments...>",
    examples = ["test", "whatever"],
    send_processing = true
)]
pub async fn default(ctxt: CommandCtxt<'_>, tag_name: Word, arguments: Option<Vec<Word>>) -> anyhow::Result<()> {
    let Some(guild_id) = ctxt.data.guild_id else {
        bail!("Tags can only be used in guilds.")
    };
    let arguments = arguments.unwrap_or_default();

    let tag = Tag::get(
        &ctxt.assyst().database_handler,
        guild_id.get() as i64,
        &tag_name.0.to_ascii_lowercase(),
    )
    .await
    .context("Failed to fetch tag")?
    .context("Tag not found in this server.")?;

    let assyst = ctxt.assyst().clone();
    let message = ctxt.data.message.cloned();
    let channel_id = ctxt.data.channel_id.get();
    let author = ctxt.data.author.clone();

    let (res, tag) = tokio::task::spawn_blocking(move || {
        let tag = tag;
        let arguments: Vec<&str> = arguments.iter().map(|Word(word)| &**word).collect();
        let tcx = TagContext {
            tokio: Handle::current(),
            message,
            assyst,
            guild_id: guild_id.get(),
            channel_id,
            author,
        };

        (
            assyst_tag::parse(&tag.data, &arguments, ParseMode::StopOnError, tcx),
            tag,
        )
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
    message: Option<Message>,
    assyst: ThreadSafeAssyst,
    guild_id: u64,
    channel_id: u64,
    author: twilight_model::user::User,
}

impl TagContext {
    fn guild_id(&self) -> u64 {
        self.guild_id
    }
}

impl assyst_tag::Context for TagContext {
    fn execute_javascript(
        &self,
        code: &str,
        args: Vec<String>,
    ) -> anyhow::Result<assyst_common::eval::FakeEvalImageResponse> {
        self.tokio
            .block_on(fake_eval(&self.assyst, code.into(), true, self.message.as_ref(), args))
    }

    fn get_last_attachment(&self) -> anyhow::Result<String> {
        let ImageUrl(attachment) = self.tokio.block_on(ImageUrl::from_channel_history(
            &self.assyst,
            Id::<ChannelMarker>::new(self.channel_id),
        ))?;
        Ok(attachment)
    }

    fn get_avatar(&self, user_id: Option<u64>) -> anyhow::Result<String> {
        let user_id = user_id.unwrap_or(self.author.id.get());

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
                true,
            ))
            .map(string_from_likely_utf8)
            .map_err(Into::into)
    }

    fn channel_id(&self) -> anyhow::Result<u64> {
        Ok(self.channel_id)
    }

    fn guild_id(&self) -> anyhow::Result<u64> {
        Ok(TagContext::guild_id(self))
    }

    fn user_id(&self) -> anyhow::Result<u64> {
        Ok(self.author.id.get())
    }

    fn user_tag(&self, id: Option<u64>) -> anyhow::Result<String> {
        if let Some(id) = id {
            self.tokio.block_on(async {
                let user = self.assyst.http_client.user(Id::new(id)).await?;
                ensure!(user.status().get() != 404, "user not found");

                Ok(format_tag(&user.model().await?))
            })
        } else {
            Ok(format_tag(&self.author))
        }
    }

    fn get_tag_contents(&self, tag: &str) -> anyhow::Result<String> {
        let tag = self
            .tokio
            .block_on(async { Tag::get(&self.assyst.database_handler, self.guild_id() as i64, tag).await });

        match tag {
            Ok(Some(Tag { data, .. })) => Ok(data),
            Ok(None) => Err(anyhow!("Tag not found")),
            Err(e) => Err(e),
        }
    }
}

define_commandgroup! {
    name: tag,
    access: Availability::Public,
    category: Category::Misc,
    aliases: ["t"],
    cooldown: Duration::from_secs(2),
    description: "assyst's tag system (documentation: https://jacher.io/tags)",
    usage: "[subcommand|tag name] <arguments...>",
    commands: [
        "create" => create,
        "edit" => edit,
        "delete" => delete,
        "list" => list,
        "info" => info,
        "raw" => raw,
        "search" => search
    ],
    default_interaction_subcommand: "run",
    default: default
}
