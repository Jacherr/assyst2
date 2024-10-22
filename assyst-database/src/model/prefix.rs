use std::hash::Hash;

use crate::DatabaseHandler;

#[derive(Clone, Hash, PartialEq)]
/// A Prefix is a unique identifier for invocating message-based commands in a guild. This table is
/// relatively simple, holding only a guild ID and its associated prefix, since each guild can only
/// have one prefix at a time.
pub struct Prefix {
    pub prefix: String,
}
impl Prefix {
    pub async fn set(&self, handler: &DatabaseHandler, guild_id: u64) -> anyhow::Result<()> {
        let query = r"INSERT INTO prefixes(guild, prefix) VALUES($1, $2) ON CONFLICT (guild) DO UPDATE SET prefix = $2 WHERE prefixes.guild = $1";

        sqlx::query(query)
            .bind(guild_id as i64)
            .bind(self.clone().prefix)
            .execute(&handler.pool)
            .await?;

        handler.cache.set_prefix(guild_id, self.clone());

        Ok(())
    }

    pub async fn get(handler: &DatabaseHandler, guild_id: u64) -> anyhow::Result<Option<Self>> {
        if let Some(prefix) = handler.cache.get_prefix(guild_id) {
            return Ok(Some(prefix));
        }

        let query = "SELECT * FROM prefixes WHERE guild = $1";

        match sqlx::query_as::<_, (String,)>(query)
            .bind(guild_id as i64)
            .fetch_one(&handler.pool)
            .await
        {
            Ok(res) => {
                let prefix = Prefix { prefix: res.0 };
                handler.cache.set_prefix(guild_id, prefix.clone());
                Ok(Some(prefix))
            },
            Err(sqlx::Error::RowNotFound) => Ok(None),
            Err(err) => Err(err.into()),
        }
    }

    #[must_use] pub fn size_of(&self) -> u64 {
        self.prefix.as_bytes().len() as u64
    }
}
