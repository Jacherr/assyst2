use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use assyst_common::config::CONFIG;
use assyst_common::err;
use assyst_common::metrics_handler::MetricsHandler;
use assyst_common::pipe::CACHE_PIPE_PATH;
use assyst_database::model::active_guild_premium_entitlement::ActiveGuildPremiumEntitlement;
use assyst_database::model::badtranslator_channel::BadTranslatorChannel;
use assyst_database::DatabaseHandler;
use assyst_flux_iface::FluxHandler;
use twilight_http::client::InteractionClient;
use twilight_http::Client as HttpClient;
use twilight_model::id::marker::ApplicationMarker;
use twilight_model::id::Id;

use crate::bad_translator::{BadTranslator, BadTranslatorEntry};
use crate::command::componentctxt::ComponentCtxts;
use crate::command_ratelimits::CommandRatelimits;
use crate::persistent_cache_handler::PersistentCacheHandler;
use crate::replies::Replies;
use crate::rest::patreon::Patron;
use crate::rest::rest_cache_handler::RestCacheHandler;
use crate::task::Task;

pub type ThreadSafeAssyst = Arc<Assyst>;

/// Main Assyst structure, storing the current bot state.
///
/// Stores stateful information and connections.
pub struct Assyst {
    /// Handler for BadTranslator channels.
    pub bad_translator: BadTranslator,
    /// Handler for the persistent assyst-cache.
    pub persistent_cache_handler: PersistentCacheHandler,
    /// Handler for the Assyst database. RwLocked to allow concurrent reads.
    pub database_handler: Arc<DatabaseHandler>,
    /// Handler for WSI.
    pub flux_handler: FluxHandler,
    /// Handler for the REST cache.
    pub rest_cache_handler: RestCacheHandler,
    /// HTTP client for Discord. Handles all HTTP requests to Discord, storing stateful information
    /// about current ratelimits.
    pub http_client: Arc<HttpClient>,
    /// Interaction client for handling Discord interations (i.e., slash commands).
    pub application_id: Id<ApplicationMarker>,
    /// List of the current premim users of Assyst.
    pub premium_users: Arc<Mutex<Vec<Patron>>>,
    /// Metrics handler for Prometheus, rate trackers etc.
    pub metrics_handler: Arc<MetricsHandler>,
    /// The reqwest client, used to issue general HTTP requests
    pub reqwest_client: reqwest::Client,
    /// Tasks are functions which are called on an interval.
    pub tasks: Mutex<Vec<Task>>,
    /// The recommended number of shards for this instance.
    pub shard_count: u64,
    /// Cached command replies for "raw" message commands.
    pub replies: Replies,
    /// All command ratelimits, in the format <(guild/user id, command name) => time command was
    /// ran>
    pub command_ratelimits: CommandRatelimits,
    /// All entitlements. At present, these entitlements are a single tier of guild subscription.
    /// `Arc`ed since it's also included as part of the Flux handler
    pub entitlements: Arc<Mutex<HashMap<i64, ActiveGuildPremiumEntitlement>>>,
    /// Component contexts, mapping a custom ID (e.g., a button) to a context.
    pub component_contexts: ComponentCtxts,
}
impl Assyst {
    pub async fn new() -> anyhow::Result<Assyst> {
        let http_client = Arc::new(HttpClient::new(CONFIG.authentication.discord_token.clone()));
        let shard_count = http_client.gateway().authed().await?.model().await?.shards as u64;
        let database_handler =
            Arc::new(DatabaseHandler::new(CONFIG.database.to_url(), CONFIG.database.to_url_safe()).await?);
        let premium_users = Arc::new(Mutex::new(vec![]));
        let current_application = http_client.current_user_application().await?.model().await?;
        let entitlements = Arc::new(Mutex::new(
            ActiveGuildPremiumEntitlement::get_all(&database_handler).await?,
        ));

        Ok(Assyst {
            bad_translator: BadTranslator::new(),
            persistent_cache_handler: PersistentCacheHandler::new(CACHE_PIPE_PATH),
            database_handler: database_handler.clone(),
            http_client: http_client.clone(),
            application_id: current_application.id,
            premium_users: premium_users.clone(),
            metrics_handler: Arc::new(MetricsHandler::new(database_handler.clone())?),
            reqwest_client: reqwest::Client::new(),
            tasks: Mutex::new(vec![]),
            shard_count,
            replies: Replies::new(),
            flux_handler: FluxHandler::new(
                database_handler.clone(),
                Arc::new(Mutex::new(HashMap::new())),
                entitlements.clone(),
            ),
            rest_cache_handler: RestCacheHandler::new(http_client.clone()),
            command_ratelimits: CommandRatelimits::new(),
            entitlements,
            component_contexts: ComponentCtxts::new(),
        })
    }

    /// Register a new `Task` to Assyst.
    pub fn register_task(&self, task: Task) {
        self.tasks.lock().unwrap().push(task);
    }

    pub fn update_premium_user_list(&self, patrons: Vec<Patron>) {
        let mut flux_prems = HashMap::new();
        for patron in &patrons {
            flux_prems.insert(patron.user_id, patron.tier as u64);
        }

        *self.premium_users.lock().unwrap() = patrons;
        self.flux_handler.set_premium_users(flux_prems);
    }

    pub fn interaction_client(&self) -> InteractionClient {
        self.http_client.interaction(self.application_id)
    }

    pub async fn init_badtranslator_channels(&self) {
        match BadTranslatorChannel::get_all(&self.database_handler).await {
            Ok(ch) => {
                let mut channels = HashMap::new();
                for c in ch {
                    channels.insert(c.id as u64, BadTranslatorEntry::with_language(c.target_language));
                }

                self.bad_translator.set_channels(channels).await;
            },
            Err(e) => {
                err!("Failed to fetch BadTranslator channels, so they will be disabled: {e:?}");
                self.bad_translator.disable().await;
            },
        }
    }
}
