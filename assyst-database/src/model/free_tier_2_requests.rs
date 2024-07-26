use crate::DatabaseHandler;

#[derive(sqlx::FromRow, Debug)]
pub struct FreeTier2Requests {
    pub user_id: i64,
    pub count: i32,
}
impl FreeTier2Requests {
    pub fn new(user_id: u64) -> FreeTier2Requests {
        FreeTier2Requests { user_id: user_id as i64, count: 0 }
    }

    pub async fn change_free_tier_2_requests(&self, handler: &DatabaseHandler, change_amount: i64) -> anyhow::Result<()> {
        let query = "insert into free_tier1_requests values($1, $2) on conflict (user_id) do update set count = free_tier1_requests.count + $2 where free_tier1_requests.user_id = $1";

        sqlx::query(query).bind(self.user_id).bind(change_amount).execute(&handler.pool).await?;

        Ok(())
    }

    pub async fn get_user_free_tier_2_requests(handler: &DatabaseHandler, user_id: u64) -> anyhow::Result<FreeTier2Requests> {
        let fetch_query = "select * from free_tier1_requests where user_id = $1";

        match sqlx::query_as::<_, FreeTier2Requests>(fetch_query).bind(user_id as i64).fetch_one(&handler.pool).await {
            Ok(x) => Ok(x),
            Err(sqlx::Error::RowNotFound) => Ok(FreeTier2Requests { user_id: user_id as i64, count: 0 }),
            Err(e) => Err(e.into()),
        }
    }
}
