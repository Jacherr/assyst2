use std::time::Instant;

use twilight_model::channel::Message;

use super::error::ParseError;
use super::preprocess::preprocess;
use crate::command::registry::find_command_by_name;
use crate::command::{ExecutionTimings, TCommand};
use crate::ThreadSafeAssyst;

pub struct ParseResult<'a> {
    pub command: TCommand,
    pub args: &'a str,
    pub calling_prefix: String,
    pub execution_timings: ExecutionTimings,
}

/// Parse any generic Message object into a Command.
///
/// This function takes all steps necessary to split a message into critical command components,
/// and if at any point the parse fails, then return with no action.
///
/// After parsing, a `CoreEvent` is fired to assyst-core signaling that the command should be
/// executed. Parsing a message has several steps.<br>
/// **Step 1**: Check if the invocating user is blacklisted. If so, prematurely return.
///
/// **Step 2**: Check that the message starts with the correct prefix.
///         The prefix can be one of four things:
///              1. The guild-specific prefix, stored in the database,
///              2. No prefix, if the command is ran in DMs,
///              3. The bot's mention, in the form of @Assyst,
///              4. The prefix override, if specified, in config.toml.
/// The mention prefix takes precedence over all other, followed by the prefix override,
/// followed by the guild prefix.         
/// This function identifies the prefix and checks if it is valid for this particular invocation.
/// If it is not, then prematurely return.
///
/// **Step 3**: Check if this Message already has an associated reply (if, for example, the
/// invocation was updated).
/// These events have a timeout for handling, to prevent editing of very old
/// messages. If it is expired, prematurely return.
///
/// **Step 4**: Parse the Command from the Message itself. If it fails to parse, prematurely return.
///
/// Once all steps are complete, a Command is returned, ready for execution.
/// Note that metadata is checked *during* execution (i.e., in the base command's `Command::execute`
/// implementation, see [`crate::command::check_metadata`])
pub async fn parse_message_into_command(
    assyst: ThreadSafeAssyst,
    message: &Message,
    processing_time_start: Instant,
    from_edit: bool,
) -> Result<Option<ParseResult>, ParseError> {
    let parse_start = Instant::now();
    let preprocess_start = Instant::now();

    let preprocess = preprocess(assyst.clone(), message, from_edit).await?;

    let preprocess_time = preprocess_start.elapsed();

    // commands can theoretically have spaces in their name so we need to try and identify the
    // set of 'words' in source text to associate with a command name (essentially finding the
    // divide between command name and command arguments)
    let command_text = &message.content[preprocess.prefix.len()..];

    let mut args = command_text.split_ascii_whitespace();
    let Some(command) = args.next() else {
        return Ok(None);
    };
    let args = args.remainder().unwrap_or("");
    let Some(command) = find_command_by_name(command) else {
        return Ok(None);
    };

    Ok(Some(ParseResult {
        command,
        args,
        calling_prefix: preprocess.prefix,
        execution_timings: ExecutionTimings {
            processing_time_start,
            parse_total: parse_start.elapsed(),
            prefix_determiner: preprocess.prefixing_determinism_time,
            metadata_check_start: Instant::now(),
            preprocess_total: preprocess_time,
        },
    }))
}
