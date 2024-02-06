use assyst_common::err;
use tracing::debug;
use twilight_model::gateway::payload::incoming::MessageCreate;

use crate::command::{CommandCtxt, CommandData};
use crate::gateway_handler::message_parser::error::{ErrorSeverity, GetErrorSeverity};
use crate::gateway_handler::message_parser::parser::parse_message_into_command;
use crate::ThreadSafeAssyst;

/// Handle a [MessageCreate] event received from the Discord gateway.
///
/// This function passes the message to the command parser, which then attempts to convert the
/// message to a command for further processing.
pub async fn handle(assyst: ThreadSafeAssyst, MessageCreate(message): MessageCreate) {
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
                    ErrorSeverity::High => err!("{err:?}"),
                }
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
