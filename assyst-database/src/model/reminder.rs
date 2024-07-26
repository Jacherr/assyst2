use std::time::{SystemTime, UNIX_EPOCH};

use crate::DatabaseHandler;

#[derive(sqlx::FromRow, Debug)]
pub struct Reminder {
    pub id: i32,
    pub user_id: i64,
    pub timestamp: i64,
    pub guild_id: i64,
    pub channel_id: i64,
    pub message_id: i64,
    pub message: String,
}
impl Reminder {
    /// Fetch all reminders with a certain maximum expiration
    pub async fn fetch_expiring_max(handler: &DatabaseHandler, time_delta: i64) -> Result<Vec<Self>, sqlx::Error> {
        let query = "SELECT * FROM reminders WHERE timestamp < $1";

        let unix: i64 = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards").as_millis().try_into().expect("count not fit u128 into target type");

        sqlx::query_as::<_, Self>(query).bind(unix + time_delta).fetch_all(&handler.pool).await
    }

    /// Fetch all reminders within a certain count for a user ID
    pub async fn fetch_user_reminders(handler: &DatabaseHandler, user: u64, count: u64) -> Result<Vec<Self>, sqlx::Error> {
        let query = r#"SELECT * FROM reminders WHERE user_id = $1 ORDER BY timestamp ASC LIMIT $2"#;

        sqlx::query_as::<_, Self>(query).bind(user as i64).bind(count as i64).fetch_all(&handler.pool).await
    }

    /// True on successful remove, false otherwise
    pub async fn remove(&self, handler: &DatabaseHandler) -> Result<bool, sqlx::Error> {
        let query = r#"DELETE FROM reminders WHERE user_id = $1 AND id = $2 RETURNING *"#;

        sqlx::query(query).bind(self.user_id).bind(self.id).fetch_all(&handler.pool).await.map(|s| !s.is_empty())
    }

    /// Add a new reminder
    pub async fn insert(&self, handler: &DatabaseHandler) -> Result<(), sqlx::Error> {
        let query = r#"INSERT INTO reminders VALUES ($1, $2, $3, $4, $5, $6)"#;

        sqlx::query(query)
            .bind(self.user_id)
            .bind(self.timestamp)
            .bind(self.guild_id)
            .bind(self.channel_id)
            .bind(self.message_id)
            .bind(&*self.message)
            .execute(&handler.pool)
            .await
            .map(|_| ())
    }
}
