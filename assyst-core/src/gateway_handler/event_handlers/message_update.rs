use tracing::debug;
use twilight_model::channel::message::MessageType;
use twilight_model::channel::Message;
use twilight_model::gateway::payload::incoming::MessageUpdate;
use twilight_model::util::Timestamp;

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
            // call command parser here
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
        .unwrap_or(Timestamp::parse("1970-01-01T01:01:01+00:00").unwrap());
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
