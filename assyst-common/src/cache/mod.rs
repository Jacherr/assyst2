use crate::pipe::Pipe;
use serde::{Deserialize, Serialize};
use tokio::spawn;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tracing::info;
use twilight_model::gateway::payload::incoming::{GuildCreate, GuildDelete, Ready};

#[derive(Serialize, Deserialize)]
pub enum CacheJob {
    HandleGuildCreate(GuildCreate),
    HandleGuildDelete(GuildDelete),
    HandleReady(Ready),
}

pub struct CacheHandler {
    pub cache_tx: UnboundedSender<CacheJob>,
}
impl CacheHandler {
    pub fn new(path: &str) -> CacheHandler {
        let (tx, rx) = unbounded_channel::<CacheJob>();
        CacheHandler::init_pipe(rx, path);
        CacheHandler { cache_tx: tx }
    }

    /// Important to consider structure here, as reader and writer are completely independent:
    /// TODO: Document reader/writer logic and handling
    fn init_pipe(rx: UnboundedReceiver<CacheJob>, path: &str) {
        let path = path.to_owned();
        spawn(async move {
            info!("Connecting to assyst-cache pipe on {path}");
            loop {
                let pipe = Pipe::poll_connect(&path, None).await.unwrap();
                info!("Connected to assyst-cache pipe on {path}");
            }
        });
    }
}
