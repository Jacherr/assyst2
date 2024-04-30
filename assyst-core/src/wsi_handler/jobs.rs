use shared::fifo::{FifoData, FifoSend};
use shared::query_params::{BloomQueryParams, CaptionQueryParams, NoneQuery, ResizeMethod, ResizeQueryParams};

use super::WsiHandler;

pub type WsiResult = anyhow::Result<Vec<u8>>;

impl WsiHandler {
    pub async fn ahshit(&self, media: Vec<u8>, user_id: u64) -> WsiResult {
        let job = FifoSend::AhShit(FifoData::new(media, NoneQuery {}));

        self.run_job(job, user_id).await
    }

    pub async fn aprilfools(&self, media: Vec<u8>, user_id: u64) -> WsiResult {
        let job = FifoSend::AprilFools(FifoData::new(media, NoneQuery {}));

        self.run_job(job, user_id).await
    }

    pub async fn bloom(&self, media: Vec<u8>, radius: usize, user_id: u64) -> WsiResult {
        let job = FifoSend::Bloom(FifoData::new(
            media,
            BloomQueryParams {
                radius,
                brightness: 35,
                sharpness: 85,
            },
        ));

        self.run_job(job, user_id).await
    }

    pub async fn caption(&self, media: Vec<u8>, text: String, user_id: u64) -> WsiResult {
        let job = FifoSend::Caption(FifoData::new(media, CaptionQueryParams { text }));

        self.run_job(job, user_id).await
    }

    pub async fn resize_absolute(&self, media: Vec<u8>, width: u32, height: u32, user_id: u64) -> WsiResult {
        let job = FifoSend::Resize(FifoData::new(
            media,
            ResizeQueryParams {
                width: Some(width),
                height: Some(height),
                method: Some(ResizeMethod::Nearest),
                scale: None,
            },
        ));

        self.run_job(job, user_id).await
    }

    pub async fn resize_scale(&self, media: Vec<u8>, scale: f32, user_id: u64) -> WsiResult {
        let job = FifoSend::Resize(FifoData::new(
            media,
            ResizeQueryParams {
                width: None,
                height: None,
                method: Some(ResizeMethod::Nearest),
                scale: Some(scale),
            },
        ));

        self.run_job(job, user_id).await
    }
}
