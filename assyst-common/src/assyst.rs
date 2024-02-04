use std::sync::Arc;

use crate::cache::CacheHandler;
use crate::config::CONFIG;
use crate::pipe::CACHE_PIPE_PATH;
use crate::prometheus::Prometheus;
use crate::task::Task;
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
    /// Tasks are functions which are called on an interval.
    pub tasks: Mutex<Vec<Task>>,
    /// Prometheus handler for graph metrics.
    pub prometheus: Mutex<Prometheus>,
    /// The reqwest client, used to issue general HTTP requests
    pub reqwest_client: reqwest::Client,
}
impl Assyst {
    pub async fn new() -> anyhow::Result<Assyst> {
        Ok(Assyst {
            cache_handler: CacheHandler::new(CACHE_PIPE_PATH),
            database_handler: RwLock::new(
                DatabaseHandler::new(CONFIG.database.to_url(), CONFIG.database.to_url_safe()).await?,
            ),
            http_client: HttpClient::new(CONFIG.authentication.discord_token.clone()),
            tasks: Mutex::new(vec![]),
            prometheus: Mutex::new(Prometheus::new()?),
            reqwest_client: reqwest::Client::new(),
        })
    }

    /// Register a new Task to Assyst.
    pub async fn register_task(&self, task: Task) {
        self.tasks.lock().await.push(task);
    }
}
