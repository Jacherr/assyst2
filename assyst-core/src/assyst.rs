use std::sync::{Arc, Mutex};

use assyst_common::config::CONFIG;
use assyst_common::metrics_handler::MetricsHandler;
use assyst_common::pipe::CACHE_PIPE_PATH;
use assyst_database::DatabaseHandler;
use twilight_http::client::InteractionClient;
use twilight_http::Client as HttpClient;
use twilight_model::id::marker::ApplicationMarker;
use twilight_model::id::Id;

use crate::command_ratelimits::CommandRatelimits;
use crate::flux_handler::FluxHandler;
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
}
impl Assyst {
    pub async fn new() -> anyhow::Result<Assyst> {
        let http_client = Arc::new(HttpClient::new(CONFIG.authentication.discord_token.clone()));
        let shard_count = http_client.gateway().authed().await?.model().await?.shards as u64;
        let database_handler =
            Arc::new(DatabaseHandler::new(CONFIG.database.to_url(), CONFIG.database.to_url_safe()).await?);
        let premium_users = Arc::new(Mutex::new(vec![]));
        let current_application = http_client.current_user_application().await?.model().await?;

        Ok(Assyst {
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
            flux_handler: FluxHandler::new(database_handler.clone(), premium_users.clone()),
            rest_cache_handler: RestCacheHandler::new(http_client.clone()),
            command_ratelimits: CommandRatelimits::new(),
        })
    }

    /// Register a new `Task` to Assyst.
    pub fn register_task(&self, task: Task) {
        self.tasks.lock().unwrap().push(task);
    }

    pub fn update_premium_user_list(&self, patrons: Vec<Patron>) {
        *self.premium_users.lock().unwrap() = patrons;
    }

    pub fn interaction_client(&self) -> InteractionClient {
        self.http_client.interaction(self.application_id)
    }
}
