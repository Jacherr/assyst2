use cache::DatabaseCache;
use sqlx::PgPool;

mod cache;
pub mod model;

/// Database hendler providing a connection to the database and helper methods for inserting,
/// fetching, deleting and modifying Assyst database data.
pub struct DatabaseHandler {
    pool: PgPool,
    cache: DatabaseCache,
}
impl DatabaseHandler {}
