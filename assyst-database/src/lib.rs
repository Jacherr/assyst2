#![feature(trait_alias)]

use std::borrow::Cow;

use cache::DatabaseCache;
use sqlx::postgres::{PgPool, PgPoolOptions};
use tracing::info;

mod cache;
pub mod model;

static MAX_CONNECTIONS: u32 = 1;

#[derive(sqlx::FromRow, Debug)]
pub struct Count {
    pub count: i64,
}

#[derive(sqlx::FromRow, Debug)]
pub struct DatabaseSize {
    pub size: String,
}

/// Database hendler providing a connection to the database and helper methods for inserting,
/// fetching, deleting and modifying Assyst database data.
pub struct DatabaseHandler {
    pool: PgPool,
    pub cache: DatabaseCache,
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

    pub async fn database_size(&self) -> anyhow::Result<DatabaseSize> {
        let query = r#"SELECT pg_size_pretty(pg_database_size('assyst')) as size"#;

        Ok(sqlx::query_as::<_, DatabaseSize>(query).fetch_one(&self.pool).await?)
    }
}

pub(crate) fn is_unique_violation(error: &sqlx::Error) -> bool {
    const UNIQUE_CONSTRAINT_VIOLATION_CODE: Cow<'_, str> = Cow::Borrowed("23505");
    error.as_database_error().and_then(|e| e.code()) == Some(UNIQUE_CONSTRAINT_VIOLATION_CODE)
}
