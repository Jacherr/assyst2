use tracing::{debug, error};
use twilight_model::gateway::payload::incoming::MessageCreate;

use crate::gateway_handler::message_parser::error::{ErrorSeverity, GetErrorSeverity};
use crate::gateway_handler::message_parser::parser::parse_message_into_command;
use crate::ThreadSafeAssyst;

/// Handle a [MessageCreate] event received from the Discord gateway.
///
/// This function passes the message to the command parser, which then attempts to convert the
/// message to a command for further processing.
pub async fn handle(assyst: ThreadSafeAssyst, event: MessageCreate) {
    match parse_message_into_command(assyst, event.0).await {
        Err(error) => {
            if error.get_severity() == ErrorSeverity::High {
                error!("{error}");
            } else {
                debug!("{error}");
            }
        },
        _ => {},
    };
}
