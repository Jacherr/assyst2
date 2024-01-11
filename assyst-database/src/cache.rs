use moka::sync::Cache;

use crate::model::prefix::Prefix;

/// In-memory cache collection for frequently accessed areas of the database.
pub struct DatabaseCache {
    prefixes: Cache<u64, Prefix>,
}
impl DatabaseCache {
    pub fn new() -> Self {
        DatabaseCache {
            prefixes: Cache::new(1000),
        }
    }
}
