use std::fmt::format;
use std::time::Instant;

use assyst_common::err;
use tracing::debug;
use twilight_model::gateway::payload::incoming::MessageCreate;

use crate::command::errors::{ExecutionError, TagParseError};
use crate::command::source::Source;
use crate::command::{CommandCtxt, CommandData};
use crate::gateway_handler::message_parser::error::{ErrorSeverity, GetErrorSeverity};
use crate::gateway_handler::message_parser::parser::parse_message_into_command;
use crate::ThreadSafeAssyst;

use super::ctxt_exec;

/// Handle a [MessageCreate] event received from the Discord gateway.
///
/// This function passes the message to the command parser, which then attempts to convert the
/// message to a command for further processing.
pub async fn handle(assyst: ThreadSafeAssyst, MessageCreate(message): MessageCreate) {
    match parse_message_into_command(assyst.clone(), &message).await {
        Ok(Some((cmd, args, calling_prefix))) => {
            let data = CommandData {
                message_id: message.id.get(),
                source: Source::Gateway,
                assyst: &assyst,
                attachment: message.attachments.first(),
                referenced_message: message.referenced_message.as_deref(),
                sticker: message.sticker_items.first(),
                channel_id: message.channel_id.get(),
                embed: message.embeds.first(),
                processing_time_start: Instant::now(),
                author: &message.author,
                calling_prefix,
                guild_id: message.guild_id.map(|x| x.get()),
            };
            let ctxt = CommandCtxt::new(args, &data);

            if let Err(err) = ctxt_exec(&ctxt, cmd).await {
                match err.get_severity() {
                    ErrorSeverity::Low => debug!("{err:?}"),
                    ErrorSeverity::High => match err {
                        // if invalid args: report usage to user
                        ExecutionError::Parse(TagParseError::ArgsExhausted) => {
                            let _ = ctxt
                                .reply(format!(
                                    ":warning: `{err}\nUsage: {}{} {}`",
                                    ctxt.data.calling_prefix,
                                    cmd.metadata().name,
                                    cmd.metadata().usage
                                ))
                                .await;
                        },
                        _ => {
                            let _ = ctxt.reply(format!(":warning: `{err}`")).await;
                        },
                    },
                }
            }

            assyst.prometheus.add_command();
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
