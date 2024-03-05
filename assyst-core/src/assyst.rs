use crate::persistent_cache_handler::PersistentCacheHandler;
use crate::replies::Replies;
use crate::rest::patreon::Patron;
use crate::rest::rest_cache_handler::RestCacheHandler;
use crate::task::Task;
use crate::wsi_handler::WsiHandler;
use assyst_common::config::CONFIG;
use assyst_common::metrics_handler::MetricsHandler;
use assyst_common::pipe::CACHE_PIPE_PATH;
use assyst_database::DatabaseHandler;
use std::sync::{Arc, Mutex};
use tokio::sync::RwLock;
use twilight_http::Client as HttpClient;

pub type ThreadSafeAssyst = Arc<Assyst>;

/// Main Assyst structure, storing the current bot state.
///
/// Stores stateful information and connections.
pub struct Assyst {
    /// Handler for the persistent assyst-cache.
    pub persistent_cache_handler: PersistentCacheHandler,
    /// Handler for the Assyst database. RwLocked to allow concurrent reads.
    pub database_handler: Arc<RwLock<DatabaseHandler>>,
    /// Handler for WSI.
    pub wsi_handler: WsiHandler,
    /// Handler for the REST cache.
    pub rest_cache_handler: RestCacheHandler,
    /// HTTP client for Discord. Handles all HTTP requests to Discord, storing stateful information
    /// about current ratelimits.
    pub http_client: Arc<HttpClient>,
    /// List of the current patrons to Assyst.
    pub patrons: Arc<Mutex<Vec<Patron>>>,
    /// Metrics handler for Prometheus, rate trackers etc.
    pub metrics_handler: Arc<MetricsHandler>,
    /// The reqwest client, used to issue general HTTP requests
    pub reqwest_client: reqwest::Client,
    /// Tasks are functions which are called on an interval.
    pub tasks: Mutex<Vec<Task>>,
    /// The recommended number of shards for this instance.
    pub shard_count: u64,
    /// Cached command replies.
    pub replies: Replies,
}
impl Assyst {
    pub async fn new() -> anyhow::Result<Assyst> {
        let http_client = Arc::new(HttpClient::new(CONFIG.authentication.discord_token.clone()));
        let shard_count = http_client.gateway().authed().await?.model().await?.shards as u64;
        let database_handler = Arc::new(RwLock::new(
            DatabaseHandler::new(CONFIG.database.to_url(), CONFIG.database.to_url_safe()).await?,
        ));
        let patrons = Arc::new(Mutex::new(vec![]));

        Ok(Assyst {
            persistent_cache_handler: PersistentCacheHandler::new(CACHE_PIPE_PATH),
            database_handler: database_handler.clone(),
            http_client: http_client.clone(),
            patrons: patrons.clone(),
            metrics_handler: Arc::new(MetricsHandler::new(database_handler.clone())?),
            reqwest_client: reqwest::Client::new(),
            tasks: Mutex::new(vec![]),
            shard_count,
            replies: Replies::new(),
            wsi_handler: WsiHandler::new(database_handler.clone(), patrons.clone()),
            rest_cache_handler: RestCacheHandler::new(http_client.clone()),
        })
    }

    /// Register a new Task to Assyst.
    pub async fn register_task(&self, task: Task) {
        self.tasks.lock().unwrap().push(task);
    }

    pub async fn update_patron_list(&self, patrons: Vec<Patron>) {
        *self.patrons.lock().unwrap() = patrons;
    }
}
