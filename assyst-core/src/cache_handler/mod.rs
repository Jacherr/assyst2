use assyst_common::cache::CacheJob;
use tokio::sync::mpsc::UnboundedSender;

pub struct CacheHandler {
    pub cache_tx: UnboundedSender<CacheJob>,
}
