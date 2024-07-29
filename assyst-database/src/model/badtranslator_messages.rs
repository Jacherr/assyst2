use crate::DatabaseHandler;

pub struct BadTranslatorMessages {
    _guild_id: i64,
    _message_count: i64,
}
impl BadTranslatorMessages {
    pub async fn increment(handler: &DatabaseHandler, guild_id: i64) -> anyhow::Result<()> {
        let query = "insert into bt_messages (guild_id, message_count) values ($1, 1) on conflict (guild_id) do update set message_count = bt_messages.message_count + 1 where bt_messages.guild_id = $1;";
        sqlx::query(query).bind(guild_id).execute(&handler.pool).await?;
        Ok(())
    }
}
