use twilight_model::gateway::payload::incoming::MessageDelete;

/// Handle a [MessageDelete] event received from the Discord gateway.
///
/// This function checks if the deleted message was one that invoked an Assyst command.
/// If it was, then Assyst will attempt to delete the response to that command, to prevent any
/// "dangling responses".
pub fn handle(event: MessageDelete) {
    // fetch the bot's response for this message if it exists, and if it does, delete it
}
