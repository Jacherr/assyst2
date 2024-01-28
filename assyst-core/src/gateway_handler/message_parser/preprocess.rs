use assyst_common::assyst::ThreadSafeAssyst;
use assyst_common::config::CONFIG;
use assyst_common::BOT_ID;
use assyst_database::model::blacklist::Blacklist;
use assyst_database::model::prefix::Prefix;
use tracing::debug;
use twilight_model::channel::Message;

use crate::gateway_handler::message_parser::error::PreParseError;

pub struct PreprocessResult {
    pub prefix: String,
}

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

/// Initial Discord message processing.
/// Checks the validity of the message before performing any kind of parsing.
///
/// This includes:
/// - Checking if the author is globally blacklisted from running commands,
/// - Checking if the author is blacklisted in the guild from running commands,
/// - Checking that the message is not sent by a bot or a webhook,
/// - Checking that the message starts with the correct prefix for the context, and returning any
///   identified prefix.
pub async fn preprocess(assyst: ThreadSafeAssyst, message: Message) -> Result<PreprocessResult, PreParseError> {
    if message.author.bot || message.webhook_id.is_some() {
        return Err(PreParseError::UserIsBotOrWebhook);
    }

    // determine which prefixes apply to this message
    // if in dm: no prefix, mention, or prefix override
    // if in guild: guild prefix, mention, or prefix override
    // if prefix override: "normal" prefix ignored
    //
    // prefix precendence:
    // 1. prefix override (disabling other prefixes)
    // 2. mention prefix
    // 3. no prefix/guild prefix (depending on context)
    let is_in_dm = message.guild_id.is_none();

    let parsed_prefix = if let Some(ref r#override) = CONFIG.dev.prefix_override {
        r#override.clone()
    } else if let Some(mention_prefix) = message_mention_prefix(&message.content) {
        mention_prefix
    } else if is_in_dm {
        "".to_owned()
    } else {
        let guild_id = message.guild_id.unwrap().get();
        let guild_prefix = Prefix::get(&mut assyst.lock().await.database_handler, guild_id).await;
        match guild_prefix {
            // found prefix in db/cache
            Ok(Some(p)) => p.prefix.clone(),
            // no prefix in db/cache, add default to db
            Ok(None) => {
                let default_prefix = Prefix {
                    prefix: CONFIG.prefix.default.clone(),
                };

                default_prefix
                    .set(&mut assyst.lock().await.database_handler, guild_id)
                    .await
                    .map_err(|e| PreParseError::Failure(format!("failed to set default prefix: {}", e.to_string())))?;

                CONFIG.prefix.default.clone()
            },
            // error fetching, throw error
            Err(error) => {
                return Err(PreParseError::Failure(format!(
                    "failed to fetch prefixes: {}",
                    error.to_string()
                )));
            },
        }
    };

    debug!("parser: parsed prefix: {:?}", parsed_prefix);

    // check blacklist second to prevent large database spam
    // from all incoming messages
    let blacklisted = Blacklist::is_blacklisted(&assyst.lock().await.database_handler, message.author.id.get()).await;
    match blacklisted {
        Ok(false) => {
            debug!(
                "parser: ignoring message: user blacklisted ({})",
                message.author.id.get()
            );
            return Err(PreParseError::UserGloballyBlacklisted);
        },
        Err(error) => {
            return Err(PreParseError::Failure(format!(
                "failed to fetch global blacklist: {}",
                error.to_string()
            )));
        },
        _ => (),
    }

    Ok(PreprocessResult { prefix: parsed_prefix })
}
