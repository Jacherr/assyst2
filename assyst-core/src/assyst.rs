use std::sync::Arc;

use crate::cache_handler::CacheHandler;
use crate::prometheus::Prometheus;
use crate::rest::patreon::Patron;
use crate::task::Task;
use assyst_common::config::CONFIG;
use assyst_common::pipe::CACHE_PIPE_PATH;
use assyst_database::DatabaseHandler;
use tokio::sync::{Mutex, RwLock};
use twilight_http::Client as HttpClient;

pub type ThreadSafeAssyst = Arc<Assyst>;

/// Main Assyst structure, storing the current bot state.
///
/// Stores stateful information and connections.
pub struct Assyst {
    /// Handler for the persistent assyst-cache.
    pub cache_handler: CacheHandler,
    /// Handler for the Assyst database. RwLocked to allow concurrent reads.
    pub database_handler: RwLock<DatabaseHandler>,
    /// HTTP client for Discord. Handles all HTTP requests to Discord, storing stateful information
    /// about current ratelimits.
    pub http_client: HttpClient,
    /// List of the current patrons to Assyst.
    pub patrons: Mutex<Vec<Patron>>,
    /// Prometheus handler for graph metrics.
    pub prometheus: Mutex<Prometheus>,
    /// The reqwest client, used to issue general HTTP requests
    pub reqwest_client: reqwest::Client,
    /// Tasks are functions which are called on an interval.
    pub tasks: Mutex<Vec<Task>>,
    /// The recommended number of shards for this instance.
    pub shard_count: u64,
}
impl Assyst {
    pub async fn new() -> anyhow::Result<Assyst> {
        let http_client = HttpClient::new(CONFIG.authentication.discord_token.clone());
        let shard_count = http_client
            .gateway()
            .authed()
            .await
            .unwrap()
            .model()
            .await
            .unwrap()
            .shards;

        Ok(Assyst {
            cache_handler: CacheHandler::new(CACHE_PIPE_PATH),
            database_handler: RwLock::new(
                DatabaseHandler::new(CONFIG.database.to_url(), CONFIG.database.to_url_safe()).await?,
            ),
            http_client,
            patrons: Mutex::new(vec![]),
            prometheus: Mutex::new(Prometheus::new()?),
            reqwest_client: reqwest::Client::new(),
            tasks: Mutex::new(vec![]),
            shard_count,
        })
    }

    /// Register a new Task to Assyst.
    pub async fn register_task(&self, task: Task) {
        self.tasks.lock().await.push(task);
    }

    pub async fn update_patron_list(&self, patrons: Vec<Patron>) {
        *self.patrons.lock().await = patrons;
    }
}
