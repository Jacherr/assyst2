use std::fmt::Display;

use assyst_common::config::CONFIG;
use assyst_common::BOT_ID;
use assyst_database::model::blacklist::Blacklist;
use assyst_database::model::prefix::Prefix;
use tracing::debug;
use twilight_model::channel::Message;

use crate::gateway_context::ThreadSafeGatewayContext;

#[derive(Debug)]
pub enum ParseError {
    PreParseFail(String),
}
impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PreParseFail(message) => {
                write!(f, "Pre-parse failed: {}", message)
            },
        }
    }
}
impl std::error::Error for ParseError {}

/// Returns `Some(prefix)` if the prefix is the mention of the bot, otherwise `None`
fn message_mention_prefix(content: &str) -> Option<String> {
    let mention_no_nickname = format!("<@{}>", BOT_ID);
    let mention_nickname = format!("<@!{}>", BOT_ID);

    if content.starts_with(&mention_no_nickname) {
        Some(mention_no_nickname)
    } else if content.starts_with(&mention_nickname) {
        Some(mention_nickname)
    } else {
        None
    }
}

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
/// Step 3: Check if this Message already has an associated reply (if, for example, the invocation
/// was updated).
/// These events have a timeout for handling, to prevent editing of very old
/// messages. If it is expired, prematurely return.
///
/// Step 4: Parse the Command from the Message itself. If it fails to parse, prematurely return.
///
/// Step 5: Using the parsed Command, identify some metadata conditionals, is the command
/// age-restricted, allowed in dms, the user has permission to use it, the cooldown
/// ratelimit isn't exceeded?
///
/// Step 6: Pass the parsed Command and its arguments to assyst-core for execution.
pub async fn parse_message_into_command(context: ThreadSafeGatewayContext, message: Message) -> Result<(), ParseError> {
    println!("a");
    let blacklisted = Blacklist::is_blacklisted(&context.lock().await.database_handler, message.author.id.get()).await;
    println!("b");
    match blacklisted {
        Ok(false) => {
            debug!(
                "parser: ignoring message: user blacklisted ({})",
                message.author.id.get()
            );
            return Ok(());
        },
        Err(error) => {
            return Err(ParseError::PreParseFail(format!(
                "failed to fetch global blacklist: {}",
                error.to_string()
            )));
        },
        _ => (),
    }

    let is_in_dm = message.guild_id.is_none();

    // determine which prefixes apply to this message
    // if in dm: no prefix, mention, or prefix override
    // if in guild: guild prefix, mention, or prefix override
    // if prefix override: "normal" prefix ignored
    //
    // prefix override takes precedence over all others
    // and disables all other prefixes
    let parsed_prefix = if let Some(ref r#override) = CONFIG.dev.prefix_override {
        r#override.clone()
    } else if let Some(mention_prefix) = message_mention_prefix(&message.content) {
        mention_prefix
    } else if is_in_dm {
        "".to_owned()
    } else {
        let guild_id = message.guild_id.unwrap().get();
        let guild_prefix = Prefix::get(&mut context.lock().await.database_handler, guild_id).await;
        match guild_prefix {
            // found prefix in db/cache
            Ok(Some(p)) => p.prefix.clone(),
            // no prefix in db/cache, add default to db
            Ok(None) => {
                let default_prefix = Prefix {
                    prefix: CONFIG.prefix.default.clone(),
                };

                default_prefix
                    .set(&mut context.lock().await.database_handler, guild_id)
                    .await
                    .map_err(|e| {
                        ParseError::PreParseFail(format!("failed to set default prefix: {}", e.to_string()))
                    })?;

                CONFIG.prefix.default.clone()
            },
            // error fetching, throw error
            Err(error) => {
                return Err(ParseError::PreParseFail(format!(
                    "failed to fetch prefixes: {}",
                    error.to_string()
                )));
            },
        }
    };

    debug!("parser: parsed prefix: {:?}", parsed_prefix);

    Ok(())
}
