use tracing::{debug, error};
use twilight_model::channel::message::MessageType;
use twilight_model::channel::Message;
use twilight_model::gateway::payload::incoming::MessageUpdate;
use twilight_model::util::Timestamp;

use crate::command::{CommandCtxt, CommandData};
use crate::gateway_handler::message_parser::error::{ErrorSeverity, GetErrorSeverity};
use crate::gateway_handler::message_parser::parser::parse_message_into_command;
use crate::ThreadSafeAssyst;

/// Handle a [MessageUpdate] event sent from the Discord gateway.
///
/// Message updates are used to check the following:
/// 1. A message was edited into a command, in which case execute that command,
/// 2. A message was edited into a different command, in which case execute the new command and edit
///    the response message,
/// 3. A command message was edited to a non-command, in which case delete the old command response.
pub async fn handle(assyst: ThreadSafeAssyst, event: MessageUpdate) {
    match convert_message_update_to_message(event) {
        Some(message) => {
            match parse_message_into_command(assyst.clone(), &message).await {
                Ok(Some((cmd, args))) => {
                    let data = CommandData {
                        assyst: &assyst,
                        attachment: message.attachments.first(),
                        referenced_message: message.referenced_message.as_deref(),
                        sticker: message.sticker_items.first(),
                        channel_id: message.channel_id.get(),
                        embed: message.embeds.first(),
                    };
                    let ctxt = CommandCtxt::new(args, &data);

                    if let Err(err) = cmd.execute(ctxt).await {
                        match err.get_severity() {
                            ErrorSeverity::Low => debug!("{err:?}"),
                            ErrorSeverity::High => error!("{err:?}"),
                        }
                    }
                },
                Ok(None) => { /* command not found */ },
                Err(error) => {
                    if error.get_severity() == ErrorSeverity::High {
                        error!("{error}");
                    } else {
                        debug!("{error}");
                    }
                },
            };
        },
        None => {},
    }
}

// yuck
fn convert_message_update_to_message(event: MessageUpdate) -> Option<Message> {
    let attachments = event.attachments.unwrap_or_default();
    let author = event.author?;
    let content = event.content.unwrap_or_default();
    let embeds = event.embeds.unwrap_or_default();
    let kind = event.kind.unwrap_or_else(|| MessageType::Regular);
    let mention_everyone = event.mention_everyone.unwrap_or_default();
    let mention_roles = event.mention_roles.unwrap_or_default();
    let pinned = event.pinned.unwrap_or_default();
    let timestamp = event
        .timestamp
        .unwrap_or_else(|| Timestamp::parse("1970-01-01T01:01:01+00:00").unwrap());
    Some(Message {
        application_id: None,
        interaction: None,
        activity: None,
        application: None,
        attachments,
        author,
        channel_id: event.channel_id,
        content,
        edited_timestamp: event.edited_timestamp,
        embeds,
        flags: None,
        guild_id: event.guild_id,
        id: event.id,
        kind,
        member: None,
        mention_channels: vec![],
        mention_everyone,
        mention_roles,
        mentions: vec![],
        pinned,
        reactions: vec![],
        reference: None,
        referenced_message: None,
        sticker_items: vec![],
        timestamp,
        tts: false,
        webhook_id: None,
        components: vec![],
        thread: None,
        role_subscription_data: None,
    })
}
