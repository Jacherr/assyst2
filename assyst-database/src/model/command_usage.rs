use crate::DatabaseHandler;

#[derive(sqlx::FromRow)]
pub struct CommandUsage {
    pub command_name: String,
    pub uses: i32,
}
impl CommandUsage {
    pub async fn get_command_usage_stats(handler: &DatabaseHandler) -> Result<Vec<Self>, sqlx::Error> {
        let query = "SELECT * FROM command_uses order by uses desc";
        sqlx::query_as::<_, Self>(query).fetch_all(&handler.pool).await
    }

    pub async fn get_command_usage_stats_for(&self, handler: &DatabaseHandler) -> Result<Self, sqlx::Error> {
        let query = "SELECT * FROM command_uses where command_name = $1 order by uses desc";
        sqlx::query_as::<_, Self>(query).bind(&self.command_name).fetch_one(&handler.pool).await
    }

    pub async fn increment_command_uses(&self, handler: &DatabaseHandler) -> Result<(), sqlx::Error> {
        let query = "insert into command_uses (command_name, uses) values ($1, 1) on conflict (command_name) do update set uses = command_uses.uses + 1 where command_uses.command_name = $1;";
        sqlx::query(query).bind(&self.command_name).execute(&handler.pool).await?;
        Ok(())
    }
}
