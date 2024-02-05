use crate::assyst::ThreadSafeAssyst;
use assyst_common::config::CONFIG;
use assyst_common::util::discord::get_guild_owner;
use assyst_common::BOT_ID;
use assyst_database::model::command_restriction::CommandRestriction;
use assyst_database::model::global_blacklist::GlobalBlacklist;
use assyst_database::model::prefix::Prefix;
use twilight_model::channel::message::MessageType;
use twilight_model::channel::Message;

use crate::gateway_handler::message_parser::error::PreParseError;

/// The resultant values from the preprocessing operation. Used later in parsing and execution.
pub struct PreprocessResult {
    /// The command prefix used in this message.
    pub prefix: String,
    /// All command restrictions for the guild the command was ran in, if any.
    pub guild_command_restrictions: Option<Vec<CommandRestriction>>,
    /// The owner of the guild, if the command was ran in a guild.
    pub guild_owner: Option<u64>,
    /// If the command was ran in the bot's DMs.
    pub is_in_dm: bool,
}

/// Returns `Some(prefix)` if the prefix is the mention of the bot, otherwise `None`
pub fn message_mention_prefix(content: &str) -> Option<String> {
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
        "".to_owned()
    } else {
        let guild_id = message.guild_id.unwrap().get();
        let guild_prefix = Prefix::get(&mut *assyst.database_handler.write().await, guild_id).await;
        match guild_prefix {
            // found prefix in db/cache
            Ok(Some(p)) => p.prefix.clone(),
            // no prefix in db/cache, add default to db
            Ok(None) => {
                let default_prefix = Prefix {
                    prefix: CONFIG.prefix.default.clone(),
                };

                default_prefix
                    .set(&mut *assyst.database_handler.write().await, guild_id)
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

    if !message.content.starts_with(&parsed_prefix) {
        return Err(PreParseError::MessageNotPrefixed(parsed_prefix));
    };

    Ok(parsed_prefix)
}

/// Checks if a user is globally blacklisted from the bot.
pub async fn user_globally_blacklisted(assyst: ThreadSafeAssyst, id: u64) -> Result<bool, PreParseError> {
    let blacklisted = GlobalBlacklist::is_blacklisted(&*assyst.database_handler.read().await, id).await;
    match blacklisted {
        Ok(x) => Ok(x),
        Err(error) => Err(PreParseError::Failure(format!(
            "failed to fetch global blacklist: {}",
            error.to_string()
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
///   identified prefix.
/// - Fetching all command restrictions for handling later once the command has been determined.
pub async fn preprocess(assyst: ThreadSafeAssyst, message: &Message) -> Result<PreprocessResult, PreParseError> {
    // check author is not bot or webhook
    if message.author.bot || message.webhook_id.is_some() {
        return Err(PreParseError::UserIsBotOrWebhook(Some(message.author.id.get())));
    }

    let relevant_message_kinds = &[MessageType::Regular, MessageType::Reply];
    if !relevant_message_kinds.contains(&message.kind) {
        return Err(PreParseError::UnsupportedMessageKind(message.kind));
    }

    let is_in_dm = message.guild_id.is_none();
    let parsed_prefix = parse_prefix(assyst.clone(), message, is_in_dm).await?;

    // check blacklist second to prevent large database spam
    // from all incoming messages
    if user_globally_blacklisted(assyst.clone(), message.author.id.get()).await? {
        return Err(PreParseError::UserGloballyBlacklisted(message.author.id.get()));
    }

    // fetch guild command restrictions and check the ones we can (any that have "all" feature
    // restriction) - server owner bypasses all restrictions so we check if user owns the server here
    let guild_owner = if !is_in_dm {
        Some(
            get_guild_owner(&assyst.http_client, message.guild_id.unwrap().get())
                .await
                .map_err(|x| PreParseError::Failure(format!("failed to get guild owner: {}", x.to_string())))?,
        )
    } else {
        None
    };

    let guild_command_restrictions = if !is_in_dm {
        Some(
            CommandRestriction::get_guild_command_restrictions(
                &*assyst.database_handler.read().await,
                message.guild_id.unwrap().get(),
            )
            .await
            .map_err(|e| {
                PreParseError::Failure(format!("failed to get guild command restrictions: {}", e.to_string()))
            })?,
        )
    } else {
        None
    };

    Ok(PreprocessResult {
        prefix: parsed_prefix,
        guild_command_restrictions,
        guild_owner,
        is_in_dm,
    })
}
