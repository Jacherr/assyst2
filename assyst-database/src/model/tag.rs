use crate::{is_unique_violation, Count, DatabaseHandler};

#[derive(sqlx::FromRow, Debug)]
pub struct Tag {
    pub name: String,
    pub data: String,
    pub author: i64,
    pub guild_id: i64,
    pub created_at: i64,
}
impl Tag {
    pub async fn get(handler: &DatabaseHandler, guild_id: i64, name: &str) -> anyhow::Result<Option<Self>> {
        let query = r#"SELECT * FROM tags WHERE name = $1 AND guild_id = $2"#;

        let result = sqlx::query_as(query)
            .bind(name)
            .bind(guild_id)
            .fetch_one(&handler.pool)
            .await;

        match result {
            Ok(v) => Ok(Some(v)),
            Err(sqlx::Error::RowNotFound) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub async fn set(&self, handler: &DatabaseHandler) -> Result<bool, sqlx::Error> {
        let query = r#"INSERT INTO tags VALUES ($1, $2, $3, $4, $5)"#;

        sqlx::query(query)
            .bind(&self.name)
            .bind(&self.data)
            .bind(self.author)
            .bind(self.guild_id)
            .bind(self.created_at)
            .execute(&handler.pool)
            .await
            .map(|_| true)
            .or_else(|e| if is_unique_violation(&e) { Ok(false) } else { Err(e) })
    }

    pub async fn delete_force(handler: &DatabaseHandler, name: &str, guild_id: i64) -> Result<bool, sqlx::Error> {
        let query = r#"DELETE FROM tags WHERE name = $1 AND guild_id = $2"#;

        sqlx::query(query)
            .bind(name)
            .bind(guild_id)
            .execute(&handler.pool)
            .await
            .map(|rows| rows.rows_affected() > 0)
            .or_else(|e| if is_unique_violation(&e) { Ok(false) } else { Err(e) })
    }

    pub async fn delete(
        handler: &DatabaseHandler,
        name: &str,
        guild_id: i64,
        author: i64,
    ) -> Result<bool, sqlx::Error> {
        let query = r#"DELETE FROM tags WHERE name = $1 AND author = $2 AND guild_id = $3"#;

        sqlx::query(query)
            .bind(name)
            .bind(author)
            .bind(guild_id)
            .execute(&handler.pool)
            .await
            .map(|rows| rows.rows_affected() > 0)
            .or_else(|e| if is_unique_violation(&e) { Ok(false) } else { Err(e) })
    }

    pub async fn edit(
        handler: &DatabaseHandler,
        author: i64,
        guild_id: i64,
        name: &str,
        new_content: &str,
    ) -> Result<bool, sqlx::Error> {
        let query = r#"UPDATE tags SET data = $1 WHERE name = $2 AND author = $3 AND guild_id = $4"#;

        sqlx::query(query)
            .bind(new_content)
            .bind(name)
            .bind(author)
            .bind(guild_id)
            .execute(&handler.pool)
            .await
            .map(|r| r.rows_affected() > 0)
    }

    pub async fn get_paged(
        handler: &DatabaseHandler,
        guild_id: i64,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<Tag>, sqlx::Error> {
        let query = r#"SELECT * FROM tags WHERE guild_id = $1 ORDER BY created_at DESC OFFSET $2 LIMIT $3"#;

        sqlx::query_as(query)
            .bind(guild_id)
            .bind(offset)
            .bind(limit)
            .fetch_all(&handler.pool)
            .await
    }

    pub async fn get_paged_for_user(
        handler: &DatabaseHandler,
        guild_id: i64,
        user_id: i64,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<Tag>, sqlx::Error> {
        let query =
            r#"SELECT * FROM tags WHERE guild_id = $1 AND author = $2 ORDER BY created_at DESC OFFSET $3 LIMIT $4"#;

        sqlx::query_as(query)
            .bind(guild_id)
            .bind(user_id)
            .bind(offset)
            .bind(limit)
            .fetch_all(&handler.pool)
            .await
    }

    pub async fn get_count_in_guild(handler: &DatabaseHandler, guild_id: i64) -> Result<i64, sqlx::Error> {
        let query = r#"SELECT count(*) FROM tags WHERE guild_id = $1"#;

        let result: Result<Count, sqlx::Error> = sqlx::query_as(query).bind(guild_id).fetch_one(&handler.pool).await;

        result.map(|c| c.count)
    }

    pub async fn get_count_for_user(
        handler: &DatabaseHandler,
        guild_id: i64,
        user_id: i64,
    ) -> Result<i64, sqlx::Error> {
        let query = r#"SELECT count(*) FROM tags WHERE guild_id = $1 AND author = $2"#;

        let result: Result<Count, sqlx::Error> = sqlx::query_as(query)
            .bind(guild_id)
            .bind(user_id)
            .fetch_one(&handler.pool)
            .await;

        result.map(|c| c.count)
    }
}
