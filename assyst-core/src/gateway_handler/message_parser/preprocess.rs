use std::time::{Duration, Instant};

use assyst_common::config::CONFIG;
use assyst_database::model::global_blacklist::GlobalBlacklist;
use assyst_database::model::prefix::Prefix;
use twilight_model::channel::message::MessageType;
use twilight_model::channel::Message;

use crate::assyst::ThreadSafeAssyst;
use crate::gateway_handler::message_parser::error::PreParseError;

/// The resultant values from the preprocessing operation. Used later in parsing and execution.
pub struct PreprocessResult {
    /// The command prefix used in this message.
    pub prefix: String,
    /// Time taken to determine the prefix.
    pub prefixing_determinism_time: Duration,
}

/// Returns `Some(prefix)` if the prefix is the mention of the bot, otherwise `None`
pub fn message_mention_prefix(content: &str) -> Option<String> {
    let mention_no_nickname = format!("<@{}>", CONFIG.bot_id);
    let mention_nickname = format!("<@!{}>", CONFIG.bot_id);

    if content.starts_with(&mention_no_nickname) {
        Some(mention_no_nickname)
    } else if content.starts_with(&mention_nickname) {
        Some(mention_nickname)
    } else {
        None
    }
}

/// Determine which prefixes apply to this message.
///
/// If in DM: no prefix, mention, or prefix override
///
/// If in guild: guild prefix, mention, or prefix override
///
/// If prefix override: "normal" prefix ignored
///
/// Prefix precendence:
/// 1. prefix override (disabling other prefixes)
/// 2. mention prefix
/// 3. no prefix/guild prefix (depending on context)
pub async fn parse_prefix(
    assyst: ThreadSafeAssyst,
    message: &Message,
    is_in_dm: bool,
) -> Result<String, PreParseError> {
    let parsed_prefix = if let Some(ref r#override) = CONFIG.dev.prefix_override
        && !r#override.is_empty()
    {
        r#override.clone()
    } else if let Some(mention_prefix) = message_mention_prefix(&message.content) {
        mention_prefix
    } else if is_in_dm {
        String::new()
    } else {
        let guild_id = message.guild_id.unwrap().get();
        let guild_prefix = Prefix::get(&assyst.database_handler, guild_id).await;
        match guild_prefix {
            // found prefix in db/cache
            Ok(Some(p)) => p.prefix,
            // no prefix in db/cache, add default to db
            Ok(None) => {
                let default_prefix = Prefix {
                    prefix: CONFIG.prefix.default.clone(),
                };

                default_prefix
                    .set(&assyst.database_handler, guild_id)
                    .await
                    .map_err(|e| PreParseError::Failure(format!("failed to set default prefix: {e}")))?;

                CONFIG.prefix.default.clone()
            },
            // error fetching, throw error
            Err(error) => {
                return Err(PreParseError::Failure(format!("failed to fetch prefixes: {error}")));
            },
        }
    };

    if !message.content.starts_with(&parsed_prefix) {
        return Err(PreParseError::MessageNotPrefixed(parsed_prefix));
    };

    Ok(parsed_prefix)
}

/// Checks if a user is globally blacklisted from the bot.
pub async fn user_globally_blacklisted(assyst: ThreadSafeAssyst, id: u64) -> Result<bool, PreParseError> {
    let blacklisted = GlobalBlacklist::is_blacklisted(&assyst.database_handler, id).await;
    match blacklisted {
        Ok(x) => Ok(x),
        Err(error) => Err(PreParseError::Failure(format!(
            "failed to fetch global blacklist: {error}",
        ))),
    }
}

/// Initial Discord message processing.
/// Checks the validity of the message before performing any kind of parsing.
///
/// This includes:
/// - Checking if the author is globally blacklisted from running commands,
/// - Checking if the message type is relavant,
/// - Checking if the author is blacklisted in the guild from running commands,
/// - Checking that the message is not sent by a bot or a webhook,
/// - Checking that the message starts with the correct prefix for the context, and returning any
///   identified prefix,
/// - Fetching all command restrictions for handling later once the command has been determined.
pub async fn preprocess(
    assyst: ThreadSafeAssyst,
    message: &Message,
    from_edit: bool,
) -> Result<PreprocessResult, PreParseError> {
    // check author is not bot or webhook
    if message.author.bot || message.webhook_id.is_some() {
        return Err(PreParseError::UserIsBotOrWebhook(Some(message.author.id.get())));
    }

    if from_edit && message.edited_timestamp.is_none() {
        return Err(PreParseError::EditedMessageWithNoTimestamp);
    }

    let relevant_message_kinds = &[MessageType::Regular, MessageType::Reply];
    if !relevant_message_kinds.contains(&message.kind) {
        return Err(PreParseError::UnsupportedMessageKind(message.kind));
    }

    let prefix_start = Instant::now();

    let is_in_dm = message.guild_id.is_none();
    let parsed_prefix = parse_prefix(assyst.clone(), message, is_in_dm).await?;

    let prefix_time = prefix_start.elapsed();

    // check blacklist second to prevent large database spam
    // from all incoming messages
    if user_globally_blacklisted(assyst.clone(), message.author.id.get()).await? {
        return Err(PreParseError::UserGloballyBlacklisted(message.author.id.get()));
    }

    Ok(PreprocessResult {
        prefix: parsed_prefix,
        prefixing_determinism_time: prefix_time,
    })
}
