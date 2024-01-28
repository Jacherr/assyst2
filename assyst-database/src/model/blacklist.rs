use crate::DatabaseHandler;

pub struct Blacklist {
    
}
impl Blacklist {
    pub async fn is_blacklisted(handler: &DatabaseHandler, user_id: u64) -> anyhow::Result<bool> {
        let query = r#"SELECT user_id FROM blacklist"#;

        match sqlx::query_as::<_, (i64,)>(query)
            .bind(user_id as i64)
            .fetch_one(&handler.pool)
            .await
            .map(|result| result.0)
        {
            Ok(_) => Ok(true),
            Err(sqlx::Error::RowNotFound) => Ok(false),
            Err(err) => Err(err.into()),
        }
    }

    pub async fn set_user_blacklisted(&self, handler: &DatabaseHandler, user_id: u64) -> anyhow::Result<()> {
        let query = r#"INSERT INTO blacklist VALUES ($1)"#;

        sqlx::query(query).bind(user_id as i64).execute(&handler.pool).await?;

        Ok(())
    }

    pub async fn remove_user_from_blacklist(&self, handler: &DatabaseHandler, user_id: u64) -> Result<(), sqlx::Error> {
        let query = r#"DELETE FROM blacklist WHERE user_id = $1"#;

        sqlx::query(query)
            .bind(user_id as i64)
            .execute(&handler.pool)
            .await
            .map(|_| ())
    }
}
