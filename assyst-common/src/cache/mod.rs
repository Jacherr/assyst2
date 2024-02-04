use std::sync::Arc;

use crate::ok_or_break;
use crate::pipe::Pipe;
use serde::{Deserialize, Serialize};
use tokio::spawn;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::sync::Mutex;
use tracing::{info, warn};
use twilight_model::gateway::payload::incoming::{GuildCreate, GuildDelete, Ready};

pub type CacheJobSend = (UnboundedSender<()>, CacheJob);

#[derive(Serialize, Deserialize)]
pub enum CacheJob {
    HandleGuildCreate(GuildCreate),
    HandleGuildDelete(GuildDelete),
    HandleReady(Ready),
}

pub struct CacheHandler {
    pub cache_tx: UnboundedSender<CacheJobSend>,
}
impl CacheHandler {
    pub fn new(path: &str) -> CacheHandler {
        let (tx, rx) = unbounded_channel::<CacheJobSend>();
        CacheHandler::init_pipe(rx, path);
        CacheHandler { cache_tx: tx }
    }

    fn init_pipe(mut rx: UnboundedReceiver<CacheJobSend>, path: &str) {
        let path = path.to_owned();
        // main handler thread
        spawn(async move {
            info!("Connecting to assyst-cache pipe on {path}");
            loop {
                loop {
                    let mut pipe = Pipe::poll_connect(&path, None).await.unwrap();
                    info!("Connected to assyst-cache pipe on {path}");

                    // ok to unwrap because tx is permanently stored in handler
                    let (tx, data) = rx.recv().await.unwrap();

                    if let Err(e) = pipe.write_object(data).await {
                        // safe to unwrap because no situation in which the channel should be dropped
                        tx.send(()).unwrap();
                        break;
                    };

                    let result = match pipe.read_object::<()>().await {
                        Ok(x) => x,
                        Err(e) => {
                            tx.send(()).unwrap();
                            break;
                        },
                    };

                    tx.send(result).unwrap();
                }
                warn!("Communication to assyst-cache lost, attempting reconnection");
            }
        });
    }
}
