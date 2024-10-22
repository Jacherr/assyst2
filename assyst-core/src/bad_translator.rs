use std::borrow::Cow;
use std::collections::HashMap;
use std::time::Duration;

use anyhow::Context;
use assyst_common::config::CONFIG;
use assyst_common::util::discord::get_avatar_url;
use assyst_common::util::{normalize_emojis, normalize_mentions, unix_timestamp};
use assyst_database::model::badtranslator_channel::BadTranslatorChannel;
use assyst_database::model::badtranslator_messages::BadTranslatorMessages;
use tokio::sync::RwLock;
use twilight_http::error::ErrorType;
use twilight_model::channel::message::{AllowedMentions, MessageType};
use twilight_model::channel::{Message, Webhook};
use twilight_model::id::marker::{ChannelMarker, UserMarker};
use twilight_model::id::Id;

use crate::assyst::Assyst;
use crate::rest::bad_translation::{bad_translate_target, TranslateError};

const BT_RATELIMIT_LEN: u64 = 5000;
const BT_RATELIMIT_MESSAGE: &str = "you are sending messages too fast.";

mod flags {
    pub const DISABLED: u32 = 0x1;
}

#[derive(Debug, Clone)]
pub struct BadTranslatorEntry {
    pub webhook: Option<Webhook>,
    pub language: Box<str>,
}

impl BadTranslatorEntry {
    pub fn with_language(language: impl Into<Box<str>>) -> Self {
        Self {
            webhook: None,
            language: language.into(),
        }
    }

    pub fn zip(self) -> Option<(Webhook, Box<str>)> {
        let language = self.language;
        self.webhook.map(|webhook| (webhook, language))
    }
}

type Snowflake = u64;
pub type ChannelCache = HashMap<Snowflake, BadTranslatorEntry>;

#[derive(Debug)]
struct BadTranslatorRatelimit(u64);

impl BadTranslatorRatelimit {
    pub fn new() -> Self {
        Self(unix_timestamp())
    }

    pub fn expired(&self) -> bool {
        unix_timestamp() - self.0 >= BT_RATELIMIT_LEN
    }
}

pub struct BadTranslator {
    flags: RwLock<u32>,
    channels: RwLock<ChannelCache>,
    ratelimits: RwLock<HashMap<u64, BadTranslatorRatelimit>>,
}

impl BadTranslator {
    pub fn new() -> Self {
        Self::with_channels(HashMap::new())
    }

    pub async fn add_channel(&self, id: Snowflake, language: &str) {
        if !self.is_disabled().await {
            let mut lock = self.channels.write().await;
            lock.insert(id, BadTranslatorEntry::with_language(language));
        }
    }

    pub async fn _set_channel_language(&self, id: u64, language: impl Into<Box<str>>) {
        let mut lock = self.channels.write().await;
        lock.entry(id).and_modify(|e| e.language = language.into());
    }

    pub fn with_channels(channels: ChannelCache) -> Self {
        Self {
            channels: RwLock::new(channels),
            ratelimits: RwLock::new(HashMap::new()),
            flags: RwLock::new(0),
        }
    }

    pub async fn is_channel(&self, k: Snowflake) -> bool {
        self.channels.read().await.contains_key(&k)
    }

    pub async fn set_channels(&self, channels: ChannelCache) {
        *self.channels.write().await = channels;
    }

    pub async fn _should_fetch(&self) -> bool {
        !self.is_disabled().await && self.channels.read().await.len() == 0
    }

    pub async fn disable(&self) {
        let mut value = self.flags.write().await;
        *value |= flags::DISABLED;
    }

    pub async fn is_disabled(&self) -> bool {
        (*self.flags.read().await & flags::DISABLED) == flags::DISABLED
    }

    pub async fn get_or_fetch_entry(&self, assyst: &Assyst, id: &Id<ChannelMarker>) -> Option<(Webhook, Box<str>)> {
        {
            // This is its own scope so that the cache lock gets dropped early
            let cache = self.channels.read().await;

            // In the perfect case where we already have the webhook cached, we can just return early
            if let Some(entry) = cache.get(&id.get()).cloned() {
                if let Some(entry) = entry.zip() {
                    return Some(entry);
                }
            }
        }

        // TODO: maybe return Result?
        let webhooks = assyst
            .http_client
            .channel_webhooks(*id)
            .await
            .ok()?
            .models()
            .await
            .ok()?;

        let webhook = webhooks.into_iter().find(|w| w.token.is_some())?;

        let mut cache = self.channels.write().await;
        let entry = cache
            .get_mut(&id.get())
            .expect("This can't fail, and if it does then that's a problem.");

        entry.webhook = Some(webhook.clone());

        Some((webhook, entry.language.clone()))
    }

    pub async fn remove_bt_channel(&self, id: u64) {
        self.channels.write().await.remove(&id);
    }

    pub async fn delete_bt_channel(&self, assyst: &Assyst, id: &Id<ChannelMarker>) -> anyhow::Result<()> {
        BadTranslatorChannel::delete(&assyst.database_handler, id.get() as i64)
            .await
            .context("Deleting BT channel failed")?;

        self.channels.write().await.remove(&id.get());
        Ok(())
    }

    /// Returns true if given user ID is ratelimited
    pub async fn try_ratelimit(&self, id: &Id<UserMarker>) -> bool {
        let mut cache = self.ratelimits.write().await;

        if let Some(entry) = cache.get(&id.get()) {
            let expired = entry.expired();

            if !expired {
                return true;
            } else {
                cache.remove(&id.get());
                return false;
            }
        }

        cache.insert(id.get(), BadTranslatorRatelimit::new());

        false
    }

    pub async fn handle_message(&self, assyst: &Assyst, message: Box<Message>) -> anyhow::Result<()> {
        // We're assuming the caller has already made sure this is a valid channel
        // So we don't check if it's a BT channel again

        let guild_id = message.guild_id.expect("There are no BT channels in DMs");

        if is_webhook(&message) || is_ratelimit_message(&message) {
            // ignore its own webhook/ratelimit messages
            return Ok(());
        }

        if message.content.is_empty()
            || message.author.bot
            || ![MessageType::Reply, MessageType::Regular].contains(&message.kind)
        {
            let _ = assyst.http_client.delete_message(message.channel_id, message.id).await;

            return Ok(());
        }

        let ratelimit = self.try_ratelimit(&message.author.id).await;
        if ratelimit {
            // delete source, respond with error, wait, delete error

            let _ = assyst.http_client.delete_message(message.channel_id, message.id).await;

            let response = assyst
                .http_client
                .create_message(message.channel_id)
                .content(&format!("<@{}>, {}", message.author.id.get(), BT_RATELIMIT_MESSAGE))
                .await?
                .model()
                .await?;

            tokio::time::sleep(Duration::from_secs(5)).await;

            assyst
                .http_client
                .delete_message(message.channel_id, response.id)
                .await?;

            return Ok(());
        }

        let content = normalize_emojis(&message.content);
        let content = normalize_mentions(&content, &message.mentions);

        let (webhook, language) = match self.get_or_fetch_entry(assyst, &message.channel_id).await {
            Some(webhook) => webhook,
            None => return self.delete_bt_channel(assyst, &message.channel_id).await,
        };

        let translation = match bad_translate_target(&assyst.reqwest_client, &content, &language).await {
            Ok(res) => Cow::Owned(res.result.text),
            Err(TranslateError::Raw(msg)) => Cow::Borrowed(msg),
            _ => return Ok(()),
        };

        let delete_state = assyst.http_client.delete_message(message.channel_id, message.id).await;

        // dont respond with translation if the source was prematurely deleted
        if let Err(ErrorType::Response {
            status,
            body: _,
            error: _,
        }) = delete_state.as_ref().map_err(twilight_http::Error::kind)
        {
            if status.get() == 404 {
                return Ok(());
            }
        }

        let token = webhook.token.as_ref().context("Failed to extract token")?;

        let translation = translation.chars().take(2000).collect::<String>();

        assyst
            .http_client
            .execute_webhook(webhook.id, token)
            .content(&translation)
            .username(&message.author.name)
            .allowed_mentions(Some(&AllowedMentions::default()))
            .avatar_url(&get_avatar_url(&message.author))
            .await
            .context("Executing webhook failed")?;

        // Increase metrics counter for this guild
        register_badtranslated_message_to_db(assyst, guild_id.get())
            .await
            .with_context(|| format!("Error updating BT message metric for {guild_id}"))?;

        Ok(())
    }
}

async fn register_badtranslated_message_to_db(assyst: &Assyst, guild_id: u64) -> anyhow::Result<()> {
    BadTranslatorMessages::increment(&assyst.database_handler, guild_id as i64).await
}

fn is_webhook(message: &Message) -> bool {
    message.author.system.unwrap_or(false) || message.webhook_id.is_some()
}

fn is_ratelimit_message(message: &Message) -> bool {
    message.content.contains(BT_RATELIMIT_MESSAGE) && message.author.id.get() == CONFIG.bot_id
}
