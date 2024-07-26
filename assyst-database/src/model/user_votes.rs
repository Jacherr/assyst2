use crate::DatabaseHandler;

/// A Voter is a user with a certain number of accrued votes. In the old Assyst, it was possible to
/// get a leaderboard of the top voters with their username and discriminator (hance these fields),
/// but these have since become unused. In a future version, they may be removed entirely.
#[derive(sqlx::FromRow, Debug)]
pub struct UserVotes {
    pub user_id: i64,
    pub username: String,
    pub discriminator: String,
    pub count: i32,
}
impl UserVotes {
    pub async fn get_user_votes(handler: &DatabaseHandler, user_id: u64) -> anyhow::Result<Option<UserVotes>> {
        let fetch_query = "select * from user_votes where user_id = $1";

        let result = sqlx::query_as::<_, UserVotes>(fetch_query).bind(user_id as i64).fetch_optional(&handler.pool).await?;

        Ok(result)
    }

    pub async fn increment_user_votes(handler: &DatabaseHandler, user_id: u64, username: &str, discriminator: &str) -> anyhow::Result<()> {
        let query = "insert into user_votes values($1, $2, $3, 1) on conflict (user_id) do update set count = user_votes.count + 1 where user_votes.user_id = $1";

        sqlx::query(query).bind(user_id as i64).bind(username).bind(discriminator).execute(&handler.pool).await?;

        Ok(())
    }
}
