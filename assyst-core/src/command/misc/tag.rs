use std::collections::HashMap;
use std::fmt::Write;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::u64;

use anyhow::{anyhow, bail, ensure, Context};
use assyst_common::util::discord::{format_discord_timestamp, format_tag, get_avatar_url};
use assyst_common::util::{string_from_likely_utf8, unix_timestamp};
use assyst_database::model::tag::Tag;
use assyst_proc_macro::command;
use assyst_string_fmt::Markdown;
use assyst_tag::parser::ParseMode;
use assyst_tag::ParseResult;
use tokio::runtime::Handle;
use twilight_model::channel::message::component::{ActionRow, ButtonStyle, TextInput, TextInputStyle};
use twilight_model::channel::message::{Component, EmojiReactionType};
use twilight_model::channel::Message;
use twilight_model::id::marker::{ChannelMarker, EmojiMarker, UserMarker};
use twilight_model::id::Id;

use super::CommandCtxt;
use crate::assyst::ThreadSafeAssyst;
use crate::command::arguments::{Image, ImageUrl, RestNoFlags, User, Word};
use crate::command::componentctxt::{
    button_emoji_new, button_new, respond_modal, respond_update_text, ComponentCtxt, ComponentInteractionData,
    ComponentMetadata,
};
use crate::command::flags::{flags_from_str, FlagDecode, FlagType};
use crate::command::messagebuilder::MessageBuilder;
use crate::command::{Availability, Category};
use crate::downloader::{download_content, ABSOLUTE_INPUT_FILE_SIZE_LIMIT_BYTES};
use crate::rest::eval::fake_eval;
use crate::{define_commandgroup, flag_parse_argument};

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
        "Tag names cannot be a reserved word."
    );
    ensure!(name.0.contains(" "), "Tag names cannot contain spaces.");

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
        name.0.to_ascii_lowercase().codestring()
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
        "Successfully deleted tag {}",
        name.0.to_ascii_lowercase().codestring()
    ))
    .await?;

    Ok(())
}

/// Used for both listing and searching tags
#[derive(Clone, Debug)]
pub struct TagPaginatorComponentMetadata {
    pub current_page: u64,
    pub page_next_cid: String,
    pub page_prev_cid: String,
    pub page_jump_cid: String,
    pub jump_modal_cid: String,
    pub jump_modal_text_cid: String,
    pub invocating_user_id: Id<UserMarker>,
    pub target_user_id: Option<Id<UserMarker>>,
    pub tag_count: u64,
    pub calling_prefix: String,
    pub search_criteria: Option<String>,
}
impl TagPaginatorComponentMetadata {
    pub async fn component_callback(&mut self, data: &ComponentInteractionData) -> anyhow::Result<()> {
        if data.invocation_user_id != self.invocating_user_id {
            bail!("This command was not ran by you.");
        }

        let pages = (self.tag_count as f64 / DEFAULT_LIST_COUNT as f64).ceil() as i64;

        // respond with modal to jump to page
        if data.custom_id == self.page_jump_cid {
            let pages = (self.tag_count as f64 / DEFAULT_LIST_COUNT as f64).ceil() as i64;
            let pages_digits = pages.to_string().len();
            respond_modal(
                data.assyst.clone(),
                data.interaction_id,
                &data.interaction_token,
                "Jump to page",
                vec![Component::ActionRow(ActionRow {
                    components: vec![Component::TextInput(TextInput {
                        custom_id: self.jump_modal_text_cid.clone(),
                        label: "Page".to_string(),
                        max_length: Some(pages_digits as u16),
                        min_length: Some(1),
                        placeholder: None,
                        required: Some(true),
                        style: TextInputStyle::Short,
                        value: None,
                    })],
                })],
                &self.jump_modal_cid,
            )
            .await?;

            return Ok(());
        } else if data.custom_id == self.jump_modal_cid {
            let modal = data.modal_submit_interaction_data.clone().context("No modal data??")?;
            let action_row = modal.components.first().context("No modal components??")?;
            let text_component = action_row
                .components
                .iter()
                .find(|c| c.custom_id == self.jump_modal_text_cid)
                .context("No page jump component??")?;
            let parsed = text_component
                .value
                .clone()
                .context("No value in text field??")?
                .parse::<u64>()
                .context("Invalid page number")?;

            if parsed > pages as u64 || parsed < 1 {
                bail!("That page doesn't exist.");
            };

            self.current_page = parsed;
        }

        if data.custom_id == self.page_next_cid {
            self.current_page += 1;
        } else if data.custom_id == self.page_prev_cid {
            self.current_page -= 1;
        };

        if self.current_page > pages as u64 {
            self.current_page = 1;
        } else if self.current_page < 1 {
            self.current_page = pages as u64;
        }

        let offset = (self.current_page as i64 - 1) * DEFAULT_LIST_COUNT;

        let tags = match self.target_user_id {
            Some(u) => match self.search_criteria {
                Some(ref s) => {
                    let all = Tag::search_in_guild_for_user(
                        &data.assyst.database_handler,
                        data.invocation_guild_id.unwrap().get() as i64,
                        u.get() as i64,
                        s,
                    )
                    .await?;
                    all[offset as usize..(offset + DEFAULT_LIST_COUNT).clamp(1, self.tag_count as i64) as usize]
                        .to_vec()
                },
                None => {
                    Tag::get_paged_for_user(
                        &data.assyst.database_handler,
                        data.invocation_guild_id.unwrap().get() as i64,
                        u.get() as i64,
                        offset,
                        DEFAULT_LIST_COUNT,
                    )
                    .await?
                },
            },
            None => match self.search_criteria {
                Some(ref s) => {
                    let all = Tag::search_in_guild(
                        &data.assyst.database_handler,
                        data.invocation_guild_id.unwrap().get() as i64,
                        s,
                    )
                    .await?;
                    all[offset as usize..(offset + DEFAULT_LIST_COUNT).clamp(1, self.tag_count as i64) as usize]
                        .to_vec()
                },
                None => {
                    Tag::get_paged(
                        &data.assyst.database_handler,
                        data.invocation_guild_id.unwrap().get() as i64,
                        offset,
                        DEFAULT_LIST_COUNT,
                    )
                    .await?
                },
            },
        };

        let mut message = format!(
            "🗒️ **Tags in this server{0}{1}**\nView a tag by running `{2}t <name>`\n\n",
            {
                match self.search_criteria {
                    Some(ref s) => format!(" with search criteria {}", s.codestring()),
                    None => "".to_owned(),
                }
            },
            {
                match self.target_user_id {
                    Some(u) => format!(" for user <@{u}>"),
                    None => "".to_owned(),
                }
            },
            self.calling_prefix,
        );

        for (index, tag) in tags.iter().enumerate() {
            let offset = (index as i64) + offset + 1;
            writeln!(
                message,
                "{}. {} {}",
                offset,
                tag.name.to_ascii_lowercase(),
                match self.target_user_id {
                    Some(_) => "".to_owned(),
                    None => format!("(<@{}>)", tag.author),
                }
            )?;
        }

        write!(
            message,
            "\nShowing {} tags (page {}/{pages}) ({} total tags)",
            tags.len(),
            self.current_page,
            self.tag_count
        )?;

        respond_update_text(
            data.assyst.clone(),
            data.interaction_id,
            &data.interaction_token,
            &message,
        )
        .await?;

        Ok(())
    }
}

#[derive(Default)]
pub struct TagListFlags {
    pub page: u64,
}
impl FlagDecode for TagListFlags {
    fn from_str(input: &str) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let mut valid_flags = HashMap::new();
        valid_flags.insert("page", FlagType::WithValue);

        let raw_decode = flags_from_str(input, valid_flags)?;
        let page = raw_decode
            .get("page")
            .and_then(|x| x.as_deref())
            .map(|x| x.parse::<u64>())
            .unwrap_or(Ok(1))
            .context("Failed to parse page number")?;

        let result = Self { page };

        Ok(result)
    }
}
flag_parse_argument! { TagListFlags }

#[command(
    description = "list tags in the server (or owned by a certain user in the server)",
    cooldown = Duration::from_secs(2),
    access = Availability::Public,
    category = Category::Misc,
    usage = "<user id|mention>",
    examples = ["@jacher"],
    flag_descriptions = [("page <page>", "start at this page number")]
)]
pub async fn list(ctxt: CommandCtxt<'_>, user: Option<User>, flags: TagListFlags) -> anyhow::Result<()> {
    let Some(guild_id) = ctxt.data.guild_id else {
        bail!("Tags can only be listed in guilds.")
    };

    let page = flags.page.clamp(1, u64::MAX);

    // user-specific search if arg is a mention
    let user_id = user.map(|x| x.0.id);

    ensure!(page >= 1, "Page must be greater or equal to 1");

    let offset = (page as i64 - 1) * DEFAULT_LIST_COUNT;
    let count = match user_id {
        Some(u) => Tag::get_count_for_user(&ctxt.assyst().database_handler, guild_id.get() as i64, u.get() as i64)
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
                u.get() as i64,
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
        "🗒️ **Tags in this server{0}**\nView a tag by running `{1}t <name>`\n\n",
        {
            match user_id {
                Some(u) => format!(" for user <@{u}>"),
                None => "".to_owned(),
            }
        },
        ctxt.data.calling_prefix,
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

    let timestamp = unix_timestamp();
    let page_next = format!("page_next-{timestamp}");
    let page_prev = format!("page_prev-{timestamp}");
    let jump_to_page = format!("page_jump-{timestamp}");
    let modal_cid = format!("page_jump-modal-{timestamp}");
    let modal_text_cid = format!("page_jump-modal-text-{timestamp}");

    ctxt.reply(MessageBuilder {
        content: Some(message),
        attachment: None,
        components: Some(vec![
            Component::Button(button_emoji_new(
                &page_prev,
                EmojiReactionType::Custom {
                    name: Some("arrow_left".to_owned()),
                    animated: false,
                    id: Id::<EmojiMarker>::new(1272681864204779560),
                },
                ButtonStyle::Secondary,
            )),
            Component::Button(button_new(&jump_to_page, "Jump", ButtonStyle::Primary)),
            Component::Button(button_emoji_new(
                &page_next,
                EmojiReactionType::Custom {
                    name: Some("arrow_right".to_owned()),
                    animated: false,
                    id: Id::<EmojiMarker>::new(1272681890129645568),
                },
                ButtonStyle::Secondary,
            )),
        ]),
        component_ctxt: Some((
            vec![
                page_next.clone(),
                page_prev.clone(),
                jump_to_page.clone(),
                modal_cid.clone(),
            ],
            ComponentCtxt::new(
                ctxt.assyst().clone(),
                ComponentMetadata::TagList(TagPaginatorComponentMetadata {
                    page_next_cid: page_next,
                    page_prev_cid: page_prev,
                    page_jump_cid: jump_to_page,
                    jump_modal_cid: modal_cid,
                    jump_modal_text_cid: modal_text_cid,
                    current_page: page,
                    invocating_user_id: ctxt.data.author.id,
                    target_user_id: user_id,
                    tag_count: count as u64,
                    calling_prefix: ctxt.data.calling_prefix.clone(),
                    search_criteria: None,
                }),
            ),
        )),
    })
    .await?;

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
pub async fn search(ctxt: CommandCtxt<'_>, query: Word, user: Option<User>) -> anyhow::Result<()> {
    let Some(guild_id) = ctxt.data.guild_id else {
        bail!("Tags can only be listed in guilds.")
    };

    // user-specific search if arg is a mention
    let user_id = user.map(|x| x.0.id);

    let page = 1;

    ensure!(page >= 1, "Page must be greater or equal to 1");

    let offset = (page as i64 - 1) * DEFAULT_LIST_COUNT;
    let tags = match user_id {
        Some(u) => {
            Tag::search_in_guild_for_user(
                &ctxt.assyst().database_handler,
                guild_id.get() as i64,
                u.get() as i64,
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

    let tags = &tags[offset as usize..(offset + DEFAULT_LIST_COUNT).clamp(1, tags.len() as i64) as usize];

    let mut message = format!(
        "🗒️ **Tags in this server with search criteria {0}{1}**\nView a tag by running `{2}t <name>`, or go to the next page by running `{2}t list {3} {0}`\n\n",
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

    let timestamp = unix_timestamp();
    let page_next = format!("page_next-{timestamp}");
    let page_prev = format!("page_prev-{timestamp}");
    let jump_to_page = format!("page_jump-{timestamp}");
    let modal_cid = format!("page_jump-modal-{timestamp}");
    let modal_text_cid = format!("page_jump-modal-text-{timestamp}");

    ctxt.reply(MessageBuilder {
        content: Some(message),
        attachment: None,
        components: Some(vec![
            Component::Button(button_emoji_new(
                &page_prev,
                EmojiReactionType::Custom {
                    name: Some("arrow_left".to_owned()),
                    animated: false,
                    id: Id::<EmojiMarker>::new(1272681864204779560),
                },
                ButtonStyle::Secondary,
            )),
            Component::Button(button_new(&jump_to_page, "Jump", ButtonStyle::Primary)),
            Component::Button(button_emoji_new(
                &page_next,
                EmojiReactionType::Custom {
                    name: Some("arrow_right".to_owned()),
                    animated: false,
                    id: Id::<EmojiMarker>::new(1272681890129645568),
                },
                ButtonStyle::Secondary,
            )),
        ]),
        component_ctxt: Some((
            vec![
                page_next.clone(),
                page_prev.clone(),
                jump_to_page.clone(),
                modal_cid.clone(),
            ],
            ComponentCtxt::new(
                ctxt.assyst().clone(),
                ComponentMetadata::TagList(TagPaginatorComponentMetadata {
                    page_next_cid: page_next,
                    page_prev_cid: page_prev,
                    page_jump_cid: jump_to_page,
                    jump_modal_cid: modal_cid,
                    jump_modal_text_cid: modal_text_cid,
                    current_page: page,
                    invocating_user_id: ctxt.data.author.id,
                    target_user_id: user_id,
                    tag_count: count as u64,
                    calling_prefix: ctxt.data.calling_prefix.clone(),
                    search_criteria: Some(query.0),
                }),
            ),
        )),
    })
    .await?;

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
        self.tokio.block_on(fake_eval(
            &self.assyst.reqwest_client,
            code.into(),
            true,
            self.message.as_ref(),
            args,
        ))
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
                &self.assyst.reqwest_client,
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
