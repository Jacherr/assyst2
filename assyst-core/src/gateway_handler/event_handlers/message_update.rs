use std::collections::HashMap;
use std::time::Instant;

use assyst_common::err;
use tracing::debug;
use twilight_model::channel::message::MessageType;
use twilight_model::channel::Message;
use twilight_model::gateway::payload::incoming::MessageUpdate;
use twilight_model::id::Id;
use twilight_model::util::Timestamp;

use crate::command::errors::{ExecutionError, TagParseError};
use crate::command::source::Source;
use crate::command::{CommandCtxt, CommandData, RawMessageParseCtxt};
use crate::gateway_handler::message_parser::error::{ErrorSeverity, GetErrorSeverity, ParseError, PreParseError};
use crate::gateway_handler::message_parser::parser::parse_message_into_command;
use crate::replies::ReplyState;
use crate::ThreadSafeAssyst;

use super::after_command_execution_success;

/// Handle a [MessageUpdate] event sent from the Discord gateway.
///
/// Message updates are used to check the following:
/// 1. A message was edited into a command, in which case execute that command,
/// 2. A message was edited into a different command, in which case execute the new command and edit
///    the response message,
/// 3. A command message was edited to a non-command, in which case delete the old command response.
pub async fn handle(assyst: ThreadSafeAssyst, event: MessageUpdate) {
    let processing_time_start = Instant::now();

    match convert_message_update_to_message(event) {
        Some(message) => {
            match parse_message_into_command(assyst.clone(), &message, processing_time_start).await {
                Ok(Some(result)) => {
                    let data = CommandData {
                        source: Source::RawMessage,
                        assyst: &assyst,
                        execution_timings: result.execution_timings,
                        calling_prefix: result.calling_prefix,
                        message: Some(&message),
                        interaction_subcommand: None,
                        channel_id: message.channel_id,
                        guild_id: message.guild_id,
                        author: message.author.clone(),
                        interaction_token: None,
                        interaction_id: None,
                        interaction_attachments: HashMap::new(),
                    };
                    let ctxt = RawMessageParseCtxt::new(CommandCtxt::new(&data), result.args);

                    if let Err(err) = result.command.execute_raw_message(ctxt.clone()).await {
                        match err.get_severity() {
                            ErrorSeverity::Low => debug!("{err:?}"),
                            ErrorSeverity::High => match err {
                                // if invalid args: report usage to user
                                ExecutionError::Parse(TagParseError::ArgsExhausted(_)) => {
                                    let _ = ctxt
                                        .cx
                                        .reply(format!(
                                            ":warning: `{err}\nUsage: {}{} {}`",
                                            ctxt.cx.data.calling_prefix,
                                            result.command.metadata().name,
                                            result.command.metadata().usage
                                        ))
                                        .await;
                                },
                                _ => {
                                    let _ = ctxt.cx.reply(format!(":warning: ``{err:#}``")).await;
                                },
                            },
                        }
                    } else {
                        after_command_execution_success(ctxt.cx, result.command);
                    }
                },
                Ok(None) | Err(ParseError::PreParseFail(PreParseError::MessageNotPrefixed(_))) => {
                    if let Some(reply) = assyst.replies.remove_raw_message(message.id.get())
                        && let ReplyState::InUse(reply) = reply.state
                    {
                        // A previous command invocation was edited to non-command, delete response
                        _ = assyst
                            .http_client
                            .delete_message(message.channel_id, Id::new(reply.message_id))
                            .await
                            .inspect_err(|err| err!("{err}"));
                    }
                },
                Err(error) => {
                    if error.get_severity() == ErrorSeverity::High {
                        err!("{error}");
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
    let kind = event.kind.unwrap_or(MessageType::Regular);
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
