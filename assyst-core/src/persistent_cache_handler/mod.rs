use assyst_common::cache::{
    CacheJob, CacheJobSend, CacheResponse, CacheResponseSend, GuildCreateData, GuildDeleteData, ReadyData,
};
use assyst_common::pipe::Pipe;
use assyst_common::unwrap_enum_variant;
use tokio::spawn;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::sync::oneshot;
use tracing::{info, warn};
use twilight_model::gateway::payload::incoming::{GuildCreate, GuildDelete, Ready};

/// Handles communication with assyst-cache.
pub struct PersistentCacheHandler {
    pub cache_tx: UnboundedSender<CacheJobSend>,
}
impl PersistentCacheHandler {
    pub fn new(path: &str) -> PersistentCacheHandler {
        let (tx, rx) = unbounded_channel::<CacheJobSend>();
        PersistentCacheHandler::init_pipe(rx, path);
        PersistentCacheHandler { cache_tx: tx }
    }

    fn init_pipe(mut rx: UnboundedReceiver<CacheJobSend>, path: &str) {
        let path = path.to_owned();
        // main handler thread
        spawn(async move {
            info!("Connecting to assyst-cache pipe on {path}");
            loop {
                let mut pipe = Pipe::poll_connect(&path, None).await.unwrap();
                info!("Connected to assyst-cache pipe on {path}");
                loop {
                    // ok to unwrap because tx is permanently stored in handler
                    let (tx, data) = rx.recv().await.unwrap();

                    if let Err(e) = pipe.write_object(data).await {
                        // safe to unwrap because no situation in which the channel should be
                        // dropped
                        tx.send(Err(e)).unwrap();
                        break;
                    };

                    let result = match pipe.read_object::<CacheResponse>().await {
                        Ok(x) => x,
                        Err(e) => {
                            tx.send(Err(e)).unwrap();
                            break;
                        },
                    };

                    tx.send(Ok(result)).unwrap();
                }
                warn!("Communication to assyst-cache lost, attempting reconnection");
            }
        });
    }

    async fn run_cache_job(&self, job: CacheJob) -> anyhow::Result<CacheResponse> {
        let (tx, rx) = oneshot::channel::<CacheResponseSend>();
        // can unwrap since it should never close
        self.cache_tx.send((tx, job)).unwrap();
        rx.await.unwrap()
    }

    /// Handles a READY event, caching its guilds. Returns the number of newly cached guilds.
    pub async fn handle_ready_event(&self, event: Ready) -> anyhow::Result<u64> {
        self.run_cache_job(CacheJob::HandleReady(ReadyData::from(event)))
            .await
            .map(|x| unwrap_enum_variant!(x, CacheResponse::NewGuildsFromReady))
    }

    /// Handles a `GUILD_CREATE`. This method returns a bool which states if this guild is new or not.
    /// A new guild is one that was not received during the start-up of the gateway connection.
    pub async fn handle_guild_create_event(&self, event: GuildCreate) -> anyhow::Result<bool> {
        self.run_cache_job(CacheJob::HandleGuildCreate(GuildCreateData::from(event)))
            .await
            .map(|x| unwrap_enum_variant!(x, CacheResponse::ShouldHandleGuildCreate))
    }

    /// Handles a `GUILD_DELETE`. This method returns a bool which states if the bot was actually
    /// kicked from this guild.
    pub async fn handle_guild_delete_event(&self, event: GuildDelete) -> anyhow::Result<bool> {
        self.run_cache_job(CacheJob::HandleGuildDelete(GuildDeleteData::from(event)))
            .await
            .map(|x| unwrap_enum_variant!(x, CacheResponse::ShouldHandleGuildDelete))
    }

    pub async fn get_guild_count(&self) -> anyhow::Result<u64> {
        let request = CacheJob::GetGuildCount;
        let response = self.run_cache_job(request).await?;
        Ok(unwrap_enum_variant!(response, CacheResponse::TotalGuilds))
    }
}
