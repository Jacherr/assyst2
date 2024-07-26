#![feature(trait_alias)]

use cache::DatabaseCache;
use sqlx::postgres::{PgPool, PgPoolOptions};
use tracing::info;

mod cache;
pub mod model;

static MAX_CONNECTIONS: u32 = 1;

#[derive(sqlx::FromRow, Debug)]
pub struct DatabaseSize {
    pub size: String,
}

#[derive(sqlx::FromRow, Debug)]
pub struct Tag {
    pub name: String,
    pub data: String,
    pub author: i64,
    pub guild_id: i64,
    pub created_at: i64,
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

    pub async fn get_tag(&self, guild_id: i64, name: &str) -> Result<Option<Tag>, sqlx::Error> {
        let query = r#"SELECT * FROM tags WHERE name = $1 AND guild_id = $2"#;

        let result = sqlx::query_as(query)
            .bind(name)
            .bind(guild_id)
            .fetch_one(&self.pool)
            .await;

        match result {
            Ok(v) => Ok(Some(v)),
            Err(sqlx::Error::RowNotFound) => Ok(None),
            Err(e) => Err(e),
        }
    }
}
