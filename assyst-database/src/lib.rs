use cache::DatabaseCache;
use sqlx::postgres::{PgPool, PgPoolOptions};

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
    pub async fn new(url: String) -> anyhow::Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(MAX_CONNECTIONS)
            .connect(&url)
            .await?;
        let cache = DatabaseCache::new();
        Ok(Self { pool, cache })
    }
}
