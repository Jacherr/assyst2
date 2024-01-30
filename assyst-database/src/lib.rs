use cache::DatabaseCache;
use sqlx::postgres::{PgPool, PgPoolOptions};
use tracing::info;

mod cache;
pub mod model;

static MAX_CONNECTIONS: u32 = 1;

/// Database hendler providing a connection to the database and helper methods for inserting,
/// fetching, deleting and modifying Assyst database data.
pub struct DatabaseHandler {
    pool: PgPool,
    cache: DatabaseCache,
}
impl DatabaseHandler {
    pub async fn new(url: String, safe_url: String) -> anyhow::Result<Self> {
        info!(
            "Connecting to database on {} with {} max connections",
            safe_url, MAX_CONNECTIONS
        );

        let pool = PgPoolOptions::new()
            .max_connections(MAX_CONNECTIONS)
            .connect(&url)
            .await?;

        info!("Connected to database on {}", safe_url);
        let cache = DatabaseCache::new();
        Ok(Self { pool, cache })
    }
}
