use crate::DatabaseHandler;

/// A colour role is a self-assignable role to grant a colour to a user.
#[derive(sqlx::FromRow, Debug)]
pub struct ColourRole {
    pub role_id: i64,
    pub name: String,
    pub guild_id: i64,
}
impl ColourRole {
    /// List all colour roles in a guild.
    pub async fn list_in_guild(handler: &DatabaseHandler, guild_id: i64) -> Result<Vec<Self>, sqlx::Error> {
        let query = r#"SELECT * FROM colors WHERE guild_id = $1"#;

        sqlx::query_as(query).bind(guild_id).fetch_all(&handler.pool).await
    }

    /// Inser a new colour role.
    pub async fn insert(&self, handler: &DatabaseHandler) -> Result<(), sqlx::Error> {
        let query = r#"INSERT INTO colors VALUES ($1, $2, $3)"#;

        sqlx::query(query)
            .bind(self.role_id)
            .bind(self.name.clone())
            .bind(self.guild_id)
            .execute(&handler.pool)
            .await
            .map(|_| ())
    }

    /// Remove a colour role. Returns true on successful removal, false if the role did not exist.
    pub async fn remove(&self, handler: &DatabaseHandler) -> Result<bool, sqlx::Error> {
        let query = r#"DELETE FROM colors WHERE guild_id = $1 AND name = $2 RETURNING *"#;

        let result = sqlx::query_as::<_, ColourRole>(query)
            .bind(self.guild_id)
            .bind(self.name.clone())
            .fetch_one(&handler.pool)
            .await;

        match result {
            Ok(_) => Ok(true),
            Err(sqlx::Error::RowNotFound) => Ok(false),
            Err(e) => Err(e),
        }
    }
}
