use assyst_common::err;
use twilight_model::gateway::payload::incoming::MessageDelete;
use twilight_model::id::Id;

use crate::assyst::ThreadSafeAssyst;
use crate::replies::ReplyState;

/// Handle a [MessageDelete] event received from the Discord gateway.
///
/// This function checks if the deleted message was one that invoked an Assyst command.
/// If it was, then Assyst will attempt to delete the response to that command, to prevent any
/// "dangling responses".
pub async fn handle(assyst: ThreadSafeAssyst, message: MessageDelete) {
    if let Some(reply) = assyst.replies.get(message.id.get())
        && let ReplyState::InUse(reply) = reply.state
    {
        _ = assyst
            .http_client
            .delete_message(message.channel_id, Id::new(reply.message_id))
            .await
            .inspect_err(|e| err!("{e}"));
    }
}
