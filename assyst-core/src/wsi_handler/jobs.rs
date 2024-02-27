use shared::fifo::{FifoData, FifoSend};
use shared::query_params::CaptionQueryParams;

use crate::assyst::ThreadSafeAssyst;

use super::WsiHandler;

impl WsiHandler {
    pub async fn caption(&self, media: Vec<u8>, text: String, user_id: u64) -> anyhow::Result<Vec<u8>> {
        let job = FifoSend::Caption(FifoData::new(media, CaptionQueryParams { text }));

        self.run_job(job, user_id).await
    }
}
