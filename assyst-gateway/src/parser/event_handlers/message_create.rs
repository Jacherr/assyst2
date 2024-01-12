use tracing::{debug, error};
use twilight_model::gateway::payload::incoming::MessageCreate;

use crate::gateway_context::ThreadSafeGatewayContext;
use crate::parser::message_parser::parse_message_into_command;

/// Handle a [MessageCreate] event received from the Discord gateway.
///
/// This function undertakes the following steps:
/// 1. Checks if the message is of interest, i.e., was not sent by a bot user. If not, it returns
///    prematurely.
/// 2. Passes the message to the command parser, which then attempts to convert the message to a
///    command for further processing.
pub async fn handle(context: ThreadSafeGatewayContext, event: MessageCreate) {
    // ignore all bot and webhook messages
    if event.author.bot || event.webhook_id.is_some() {
        debug!(
            "MESSAGE_CREATE: message not of interest: {} author",
            if event.author.bot { "bot" } else { "webhook" }
        );
        return;
    }

    match parse_message_into_command(context, event.0).await {
        Err(error) => {
            error!("error: {}", error);
        },
        _ => {},
    };
}
