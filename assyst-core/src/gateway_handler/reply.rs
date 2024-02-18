use std::time::Instant;

use twilight_model::channel::message::AllowedMentions;
use twilight_model::http::attachment::Attachment as TwilightAttachment;
use twilight_model::id::Id;

use crate::command::messagebuilder::MessageBuilder;
use crate::command::CommandCtxt;
use crate::replies::{Reply, ReplyInUse, ReplyState};

/// Trims a `String` in-place such that it fits in Discord's 2000 character message limit.
fn trim_content_fits(content: &mut String) {
    if let Some((truncated_byte_index, _)) = content.char_indices().nth(2000) {
        // If the content length exceeds 2000 characters, truncate it at the 2000th characters' byte index
        content.truncate(truncated_byte_index);
    }
}

pub async fn edit(ctxt: &CommandCtxt<'_>, mut builder: MessageBuilder, reply: ReplyInUse) -> anyhow::Result<()> {
    // If either the already-sent reply or the builder has an
    // attachment, we need to delete the reply and send another one, since an attachment can't be
    // edited.
    if builder.attachment.is_some() || reply.has_attachments {
        ctxt.data
            .assyst
            .http_client
            .delete_message(Id::new(ctxt.data.channel_id), Id::new(reply.message_id))
            .await?;

        return create_message(ctxt, builder).await;
    }

    let allowed_mentions = AllowedMentions::default();

    let mut message = ctxt
        .data
        .assyst
        .http_client
        .update_message(Id::new(ctxt.data.channel_id), Id::new(reply.message_id))
        .allowed_mentions(Some(&allowed_mentions));

    if let Some(content) = &mut builder.content {
        trim_content_fits(content);
        message = message.content(Some(content))?;
    }

    message.await?;
    Ok(())
}

async fn create_message(ctxt: &CommandCtxt<'_>, mut builder: MessageBuilder) -> anyhow::Result<()> {
    let allowed_mentions = AllowedMentions::default();

    let mut message = ctxt
        .data
        .assyst
        .http_client
        .create_message(Id::new(ctxt.data.channel_id))
        .allowed_mentions(Some(&allowed_mentions));

    if let Some(content) = &mut builder.content {
        trim_content_fits(content);
        message = message.content(content)?;
    }

    let attachments;
    if let Some(attachment) = builder.attachment {
        attachments = [TwilightAttachment::from_bytes(
            attachment.name.into(),
            attachment.data,
            0,
        )];

        message = message.attachments(&attachments)?;
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
