use crate::{is_unique_violation, DatabaseHandler};

#[derive(sqlx::FromRow)]
pub struct BadTranslatorChannel {
    pub id: i64,
    pub target_language: String,
}
impl BadTranslatorChannel {
    pub async fn get_all(handler: &DatabaseHandler) -> anyhow::Result<Vec<Self>> {
        let query = "SELECT * FROM bt_channels";
        let rows = sqlx::query_as::<_, Self>(query).fetch_all(&handler.pool).await?;
        Ok(rows)
    }

    pub async fn delete(handler: &DatabaseHandler, id: i64) -> anyhow::Result<bool> {
        let query = r#"DELETE FROM bt_channels WHERE id = $1 RETURNING *"#;

        Ok(sqlx::query(query)
            .bind(id)
            .fetch_all(&handler.pool)
            .await
            .map(|x| !x.is_empty())?)
    }

    pub async fn update_language(&self, handler: &DatabaseHandler, new_language: &str) -> anyhow::Result<bool> {
        let query = r#"UPDATE bt_channels SET target_language = $1 WHERE id = $2 RETURNING *"#;

        Ok(sqlx::query(query)
            .bind(new_language)
            .bind(self.id)
            .fetch_all(&handler.pool)
            .await
            .map(|x| !x.is_empty())?)
    }

    pub async fn set(&self, handler: &DatabaseHandler) -> anyhow::Result<bool> {
        let query = r#"INSERT INTO bt_channels VALUES ($1, $2)"#;

        sqlx::query(query)
            .bind(self.id)
            .bind(&self.target_language)
            .execute(&handler.pool)
            .await
            .map(|_| true)
            .or_else(|e| {
                if is_unique_violation(&e) {
                    Ok(false)
                } else {
                    Err(e.into())
                }
            })
    }
}
