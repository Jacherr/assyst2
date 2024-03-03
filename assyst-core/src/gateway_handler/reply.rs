use std::time::Instant;

use assyst_common::util::filetype::{get_sig, Type};
use twilight_model::channel::message::AllowedMentions;
use twilight_model::http::attachment::Attachment as TwilightAttachment;
use twilight_model::id::Id;

use crate::command::messagebuilder::MessageBuilder;
use crate::command::CommandCtxt;
use crate::replies::{Reply, ReplyInUse, ReplyState};
use crate::rest::filer::upload_to_filer;
use crate::rest::NORMAL_DISCORD_UPLOAD_LIMIT_BYTES;

/// Trims a `String` in-place such that it fits in Discord's 2000 character message limit.
fn trim_content_fits(content: &mut String) {
    if let Some((truncated_byte_index, _)) = content.char_indices().nth(2000) {
        // If the content length exceeds 2000 characters, truncate it at the 2000th characters' byte index
        content.truncate(truncated_byte_index);
    }
}

/// Gets the Filer URL for this attachment if it exceeds the guild's upload limit.
async fn get_filer_url(
    ctxt: &CommandCtxt<'_>,
    content: Option<&String>,
    data: Vec<u8>,
) -> anyhow::Result<Option<String>> {
    let filer_url;
    let filer_formatted;

    if data.len() > NORMAL_DISCORD_UPLOAD_LIMIT_BYTES as usize {
        let guild_upload_limit = if let Some(guild_id) = ctxt.data.guild_id {
            ctxt.assyst()
                .rest_cache_handler
                .get_guild_upload_limit_bytes(guild_id)
                .await?
        } else {
            NORMAL_DISCORD_UPLOAD_LIMIT_BYTES
        };

        if data.len() > guild_upload_limit as usize {
            filer_url = upload_to_filer(
                ctxt.assyst().clone(),
                data.clone(),
                get_sig(&data).unwrap_or(Type::PNG).as_mime(),
            )
            .await?;

            if let Some(content) = content {
                filer_formatted = format!("{content} {filer_url}");
                return Ok(Some(filer_formatted));
            }
            return Ok(Some(filer_url));
        }
        return Ok(None);
    }
    Ok(None)
}

pub async fn edit(ctxt: &CommandCtxt<'_>, builder: MessageBuilder, reply: ReplyInUse) -> anyhow::Result<()> {
    let allowed_mentions = AllowedMentions::default();

    let mut message = ctxt
        .data
        .assyst
        .http_client
        .update_message(Id::new(ctxt.data.channel_id), Id::new(reply.message_id))
        .allowed_mentions(Some(&allowed_mentions));

    let mut content_clone = builder.content.clone();

    if builder.attachment.is_none() && builder.content.as_ref().map(|x| x.trim().is_empty()).unwrap_or(true) {
        message = message.content(Some("[Empty Response]"))?;
    } else if let Some(content) = &mut content_clone {
        trim_content_fits(content);
        message = message.content(Some(content))?;
    }

    let attachments;
    let url;
    if let Some(attachment) = builder.attachment {
        if let Some(found_url) = get_filer_url(ctxt, builder.content.as_ref(), attachment.data.clone()).await? {
            url = found_url;
            message = message.content(Some(&url))?;
        } else {
            attachments = [TwilightAttachment::from_bytes(
                attachment.name.into(),
                attachment.data,
                0,
            )];
            message = message.attachments(&attachments)?;
            if builder.content.is_none() {
                message = message.content(Some(""))?;
            }
        };
    }

    message.await?;
    Ok(())
}

async fn create_message(ctxt: &CommandCtxt<'_>, builder: MessageBuilder) -> anyhow::Result<()> {
    let allowed_mentions = AllowedMentions::default();

    let mut message = ctxt
        .data
        .assyst
        .http_client
        .create_message(Id::new(ctxt.data.channel_id))
        .allowed_mentions(Some(&allowed_mentions));

    let mut content_clone = builder.content.clone();

    if builder.attachment.is_none() && builder.content.as_ref().map(|x| x.trim().is_empty()).unwrap_or(true) {
        message = message.content("[Empty Response]")?;
    } else if let Some(content) = &mut content_clone {
        trim_content_fits(content);
        message = message.content(content)?;
    }

    let attachments;
    let url;
    if let Some(attachment) = builder.attachment {
        if let Some(found_url) = get_filer_url(ctxt, builder.content.as_ref(), attachment.data.clone()).await? {
            url = found_url;
            message = message.content(&url)?;
        } else {
            attachments = [TwilightAttachment::from_bytes(
                attachment.name.into(),
                attachment.data,
                0,
            )];
            message = message.attachments(&attachments)?;
            if builder.content.is_none() {
                message = message.content("")?;
            }
        };
    }

    let reply = message.await?.model().await?;
    ctxt.data.assyst.replies.insert(
        ctxt.data.message_id,
        Reply {
            state: ReplyState::InUse(ReplyInUse {
                message_id: reply.id.get(),
                has_attachments: !reply.attachments.is_empty(),
            }),
            created: Instant::now(),
        },
    );

    Ok(())
}

pub async fn reply(ctxt: &CommandCtxt<'_>, builder: MessageBuilder) -> anyhow::Result<()> {
    let reply_in_use = ctxt
        .data
        .assyst
        .replies
        .get(ctxt.data.message_id)
        .and_then(|r| r.in_use());

    if let Some(reply_in_use) = reply_in_use {
        edit(ctxt, builder, reply_in_use).await
    } else {
        create_message(ctxt, builder).await
    }
}
