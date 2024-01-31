use std::sync::Arc;

use crate::config::CONFIG;
use assyst_database::DatabaseHandler;
use tokio::sync::Mutex;
use twilight_http::Client as HttpClient;

pub type ThreadSafeAssyst = Arc<Mutex<Assyst>>;

/// Main Assyst structure, storing the current bot state.
///
/// Stores stateful information and connections.
pub struct Assyst {
    pub database_handler: DatabaseHandler,
    pub http_client: HttpClient,
}
impl Assyst {
    pub async fn new() -> Option<Assyst> {
        Some(Assyst {
            database_handler: DatabaseHandler::new(CONFIG.database.to_url(), CONFIG.database.to_url_safe())
                .await
                .ok()?,
            http_client: HttpClient::new(CONFIG.authentication.discord_token.clone()),
        })
    }
}
