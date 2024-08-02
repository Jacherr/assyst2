use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{is_unique_violation, DatabaseHandler};

#[derive(sqlx::FromRow, Clone)]
pub struct ActiveGuildPremiumEntitlement {
    pub entitlement_id: i64,
    pub guild_id: i64,
    pub user_id: i64,
    pub started_unix_ms: i64,
    pub expiry_unix_ms: i64,
}
impl ActiveGuildPremiumEntitlement {
    pub async fn set(&self, handler: &DatabaseHandler) -> anyhow::Result<bool> {
        let query = r#"INSERT INTO active_guild_premium_entitlements VALUES ($1, $2, $3, $4, $5)"#;

        Ok(sqlx::query(query)
            .bind(self.entitlement_id)
            .bind(self.guild_id)
            .bind(self.user_id)
            .bind(self.started_unix_ms)
            .bind(self.expiry_unix_ms)
            .execute(&handler.pool)
            .await
            .map(|_| true)
            .or_else(|e| if is_unique_violation(&e) { Ok(false) } else { Err(e) })?)
    }

    pub async fn delete(handler: &DatabaseHandler, entitlement_id: i64) -> anyhow::Result<()> {
        let query = r#"DELETE FROM active_guild_premium_entitlements WHERE entitlement_id = $1"#;
        sqlx::query(query).bind(entitlement_id).execute(&handler.pool).await?;

        Ok(())
    }

    /// Useful on ENTITLEMENT_UPDATE where the user got billed and the expiry changes
    pub async fn update(&self, handler: &DatabaseHandler) -> anyhow::Result<bool> {
        let query = r#"UPDATE active_guild_premium_entitlements SET guild_id = $2, user_id = $3, started_unix_ms = $4, expiry_unix_ms = $5 WHERE entitlement_id = $1"#;

        Ok(sqlx::query(query)
            .bind(self.entitlement_id)
            .bind(self.guild_id)
            .bind(self.user_id)
            .bind(self.started_unix_ms)
            .bind(self.expiry_unix_ms)
            .execute(&handler.pool)
            .await
            .map(|_| true)
            .or_else(|e| if is_unique_violation(&e) { Ok(false) } else { Err(e) })?)
    }

    pub async fn get_all(handler: &DatabaseHandler) -> anyhow::Result<HashMap<i64, Self>> {
        let query = "SELECT * FROM active_guild_premium_entitlements";
        let rows = sqlx::query_as::<_, Self>(query).fetch_all(&handler.pool).await?;
        let mut out = HashMap::new();
        for r in rows {
            out.insert(r.guild_id, r);
        }

        Ok(out)
    }

    pub fn expired(&self) -> bool {
        let current = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
        current > self.expiry_unix_ms as u128
    }
}
