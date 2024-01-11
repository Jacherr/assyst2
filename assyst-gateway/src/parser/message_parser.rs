use assyst_common::gateway::core_event::CoreEventSender;
use twilight_model::channel::Message;

use crate::gateway_context::ThreadSafeGatewayContext;

/// Parse any generic Message object into a Command.
///
/// This function takes all steps necessary to split a message into critical command components,
/// and if at any point the parse fails, then return with no action.
///
/// After parsing, a CoreEvent is fired to assyst-core signaling that the command should be
/// executed. Parsing a message has several steps.<br>
/// Step 1: Check if the invocating user is blacklisted. If so, prematurely return.
///
/// Step 2: Check that the message starts with the correct prefix.
///         The prefix can be one of three things:
///              1. The guild-specific prefix, stored in the database,
///              2. The bot's mention, in the form of @Assyst,
///              3. The prefix override, if specified, in config.toml.
///         The mention prefix takes precedence over all other, followed by the prefix override,
/// followed by the guild prefix.         This function identifies the prefix and checks if it is
/// valid for this particular invocation.         If it is not, then prematurely return.
///
/// Step 3: Check if this Message already has an associated reply (if, for example, the invocation
/// was updated).         These events have a timeout for handling, to prevent editing of very old
/// messages. If it is expired,         prematurely return.
///
/// Step 4: Parse the Command from the Message itself. If it fails to parse, prematurely return.
///
/// Step 5: Using the parsed Command, identify some metadata conditionals, is the command
/// age-restricted,         allowed in dms, the user has permission to use it, the cooldown
/// ratelimit isn't exceeded?
///
/// Step 6: Pass the parsed Command and its arguments to assyst-core for execution.
pub fn parse_message_into_command(context: ThreadSafeGatewayContext, message: Message) -> anyhow::Result<()> {
    Ok(())
}
