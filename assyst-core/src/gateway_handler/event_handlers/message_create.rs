use std::collections::HashMap;
use std::time::Instant;

use assyst_common::err;
use tracing::debug;
use twilight_model::gateway::payload::incoming::MessageCreate;

use crate::command::errors::{ExecutionError, TagParseError};
use crate::command::source::Source;
use crate::command::{CommandCtxt, CommandData, RawMessageParseCtxt};
use crate::gateway_handler::message_parser::error::{ErrorSeverity, GetErrorSeverity};
use crate::gateway_handler::message_parser::parser::parse_message_into_command;
use crate::ThreadSafeAssyst;

use super::after_command_execution_success;

/// Handle a [MessageCreate] event received from the Discord gateway.
///
/// This function passes the message to the command parser, which then attempts to convert the
/// message to a command for further processing.
pub async fn handle(assyst: ThreadSafeAssyst, MessageCreate(message): MessageCreate) {
    let processing_time_start = Instant::now();

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
        Ok(None) => { /* command not found */ },
        Err(error) => {
            if error.get_severity() == ErrorSeverity::High {
                err!("{error}");
            } else {
                debug!("{error}");
            }
        },
    };
}
