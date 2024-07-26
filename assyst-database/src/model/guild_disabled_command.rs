use anyhow::Context;

use crate::DatabaseHandler;

#[derive(sqlx::FromRow)]
pub struct GuildDisabledCommand {
    pub guild_id: i64,
    pub command_name: String,
}
impl GuildDisabledCommand {
    pub async fn is_disabled(&self, handler: &DatabaseHandler) -> anyhow::Result<bool> {
        if let Some(commands) = handler.cache.get_guild_disabled_commands(self.guild_id as u64) {
            return Ok(commands.lock().unwrap().contains(&self.command_name));
        }

        let query = "select * from disabled_commands where guild_id = $1";
        let result = sqlx::query_as::<_, Self>(query).bind(self.guild_id).fetch_all(&handler.pool).await.context("Failed to fetch guild disabled commands from database")?;

        handler.cache.reset_disabled_commands_for(self.guild_id as u64);

        for command in &result {
            handler.cache.set_command_disabled(self.guild_id as u64, &command.command_name);
        }

        let is_disabled = result.iter().any(|cmd| cmd.command_name == self.command_name);
        Ok(is_disabled)
    }

    pub async fn enable(&self, handler: &DatabaseHandler) -> anyhow::Result<()> {
        let query = "delete from disabled_commands where guild_id = $1 and command_name = $2";

        sqlx::query(query).bind(self.guild_id).bind(&self.command_name).execute(&handler.pool).await?;

        handler.cache.set_command_enabled(self.guild_id as u64, &self.command_name);

        Ok(())
    }

    pub async fn disable(&self, handler: &DatabaseHandler) -> anyhow::Result<()> {
        let query = "insert into disabled_commands(guild_id, command_name) values($1, $2)";

        sqlx::query(query).bind(self.guild_id).bind(&self.command_name).execute(&handler.pool).await?;

        handler.cache.set_command_disabled(self.guild_id as u64, &self.command_name);

        Ok(())
    }
}
