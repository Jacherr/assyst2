use crate::DatabaseHandler;

/// The global blacklist is a list of users who are completely blacklisted from using any of the
/// bot's functionality. It is a simple list table with one column of user IDs which are
/// blacklisted.
pub struct GlobalBlacklist {}
impl GlobalBlacklist {
    pub async fn is_blacklisted(handler: &DatabaseHandler, user_id: u64) -> anyhow::Result<bool> {
        if let Some(blacklisted) = handler.cache.get_user_global_blacklist(user_id) {
            return Ok(blacklisted);
        }

        let query = r"SELECT user_id FROM blacklist WHERE user_id = $1";

        match sqlx::query_as::<_, (i64,)>(query)
            .bind(user_id as i64)
            .fetch_one(&handler.pool)
            .await
            .map(|result| result.0)
        {
            Ok(_) => {
                handler.cache.set_user_global_blacklist(user_id, true);
                Ok(true)
            },
            Err(sqlx::Error::RowNotFound) => {
                handler.cache.set_user_global_blacklist(user_id, false);
                Ok(false)
            },
            Err(err) => Err(err.into()),
        }
    }

    pub async fn set_user_blacklisted(&self, handler: &DatabaseHandler, user_id: u64) -> anyhow::Result<()> {
        let query = r"INSERT INTO blacklist VALUES ($1)";

        sqlx::query(query).bind(user_id as i64).execute(&handler.pool).await?;
        handler.cache.set_user_global_blacklist(user_id, true);

        Ok(())
    }

    pub async fn remove_user_from_blacklist(&self, handler: &DatabaseHandler, user_id: u64) -> Result<(), sqlx::Error> {
        let query = r"DELETE FROM blacklist WHERE user_id = $1";

        handler.cache.set_user_global_blacklist(user_id, false);
        sqlx::query(query)
            .bind(user_id as i64)
            .execute(&handler.pool)
            .await
            .map(|_| ())
    }
}
