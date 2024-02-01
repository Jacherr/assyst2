use std::sync::Arc;

use crate::config::CONFIG;
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
    pub database_handler: RwLock<DatabaseHandler>,
    pub http_client: HttpClient,
    pub tasks: Mutex<Vec<Task>>,
    pub prometheus: Mutex<Prometheus>,
}
impl Assyst {
    pub async fn new() -> anyhow::Result<Assyst> {
        Ok(Assyst {
            database_handler: RwLock::new(
                DatabaseHandler::new(CONFIG.database.to_url(), CONFIG.database.to_url_safe()).await?,
            ),
            http_client: HttpClient::new(CONFIG.authentication.discord_token.clone()),
            tasks: Mutex::new(vec![]),
            prometheus: Mutex::new(Prometheus::new()?),
        })
    }

    pub async fn register_task(&self, task: Task) {
        self.tasks.lock().await.push(task);
    }
}
