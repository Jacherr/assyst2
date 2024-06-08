use crate::DatabaseHandler;
use std::time::{SystemTime, UNIX_EPOCH};

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
    pub async fn fetch_expiring_max(handler: &DatabaseHandler, time_delta: i64) -> Result<Vec<Self>, sqlx::Error> {
        let query = "SELECT * FROM reminders WHERE timestamp < $1";

        let unix: i64 = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis()
            .try_into()
            .expect("count not fit u128 into target type");

        sqlx::query_as::<_, Self>(query)
            .bind(unix + time_delta)
            .fetch_all(&handler.pool)
            .await
    }

    /// True on successful remove, false otherwise
    pub async fn remove(&self, handler: &DatabaseHandler) -> Result<bool, sqlx::Error> {
        let query = r#"DELETE FROM reminders WHERE user_id = $1 AND id = $2 RETURNING *"#;

        sqlx::query(query)
            .bind(self.user_id as i64)
            .bind(self.id)
            .fetch_all(&handler.pool)
            .await
            .map(|s| !s.is_empty())
    }
}
