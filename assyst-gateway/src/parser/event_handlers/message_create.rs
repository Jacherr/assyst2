use assyst_common::gateway::core_event::CoreEventSender;
use tracing::debug;
use twilight_model::gateway::payload::incoming::MessageCreate;

use crate::gateway_context::ThreadSafeGatewayContext;

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

    // call command parser here
}
