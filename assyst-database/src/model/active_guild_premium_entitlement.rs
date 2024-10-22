use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::bail;
use twilight_model::application::monetization::Entitlement;
use twilight_model::util::Timestamp;

use crate::{is_unique_violation, DatabaseHandler};

#[derive(sqlx::FromRow, Clone, Debug)]
pub struct ActiveGuildPremiumEntitlement {
    pub entitlement_id: i64,
    pub guild_id: i64,
    pub user_id: i64,
    pub started_unix_ms: i64,
    pub expiry_unix_ms: i64,
}
impl ActiveGuildPremiumEntitlement {
    pub async fn set(&self, handler: &DatabaseHandler) -> anyhow::Result<bool> {
        let query = r"INSERT INTO active_guild_premium_entitlements VALUES ($1, $2, $3, $4, $5)";

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
        let query = r"DELETE FROM active_guild_premium_entitlements WHERE entitlement_id = $1";
        sqlx::query(query).bind(entitlement_id).execute(&handler.pool).await?;

        Ok(())
    }

    /// Useful on `ENTITLEMENT_UPDATE` where the user got billed and the expiry changes
    pub async fn update(&self, handler: &DatabaseHandler) -> anyhow::Result<bool> {
        let query = r"UPDATE active_guild_premium_entitlements SET guild_id = $2, user_id = $3, started_unix_ms = $4, expiry_unix_ms = $5 WHERE entitlement_id = $1";

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
            out.insert(r.entitlement_id, r);
        }

        Ok(out)
    }

    #[must_use] pub fn expired(&self) -> bool {
        let current = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
        current > self.expiry_unix_ms as u128 && self.expiry_unix_ms != 0
    }
}
impl TryFrom<Entitlement> for ActiveGuildPremiumEntitlement {
    type Error = anyhow::Error;

    fn try_from(value: Entitlement) -> Result<Self, Self::Error> {
        let Some(guild_id) = value.guild_id else {
            bail!(
                "Entitlement ID {} (guild {:?} user {:?}) has no associated guild!",
                value.id,
                value.guild_id,
                value.user_id
            )
        };

        let Some(user_id) = value.user_id else {
            bail!(
                "Entitlement ID {} (guild {:?} user {:?}) has no associated user!",
                value.id,
                value.guild_id,
                value.user_id
            )
        };

        // no expiry/created = test entitlement, requires special handling
        let active = Self {
            entitlement_id: value.id.get() as i64,
            guild_id: guild_id.get() as i64,
            user_id: user_id.get() as i64,
            started_unix_ms: value
                .starts_at
                .unwrap_or(Timestamp::from_micros(0).unwrap())
                .as_micros()
                / 1000,
            expiry_unix_ms: value.ends_at.unwrap_or(Timestamp::from_micros(0).unwrap()).as_micros() / 1000,
        };

        Ok(active)
    }
}
