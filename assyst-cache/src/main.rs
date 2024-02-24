use assyst_common::cache::{CacheJob, CacheResponse};
use assyst_common::ok_or_break;
use assyst_common::pipe::pipe_server::PipeServer;
use assyst_common::pipe::CACHE_PIPE_PATH;
use assyst_common::util::tracing_init;
use tracing::{debug, info, warn};

use crate::caches::guilds::GuildCache;

mod caches;

#[tokio::main]
async fn main() {
    tracing_init();

    let mut guild_cache = GuildCache::new();

    let mut pipe_server = PipeServer::listen(CACHE_PIPE_PATH).unwrap();
    info!("Awaiting connection from assyst-core");
    loop {
        let mut stream = pipe_server.accept_connection().await.unwrap();
        info!("Connection received from assyst-core");
        loop {
            let job = ok_or_break!(stream.read_object::<CacheJob>().await);

            debug!("Handling job: {}", job);

            let result = match job {
                CacheJob::HandleReady(event) => {
                    CacheResponse::NewGuildsFromReady(guild_cache.handle_ready_event(event))
                },
                CacheJob::HandleGuildCreate(event) => {
                    CacheResponse::ShouldHandleGuildCreate(guild_cache.handle_guild_create_event(event))
                },
                CacheJob::HandleGuildDelete(event) => {
                    CacheResponse::ShouldHandleGuildDelete(guild_cache.handle_guild_delete_event(event))
                },
            };

            ok_or_break!(stream.write_object(result).await);
        }
        warn!("Connection to assyst-core lost, awaiting reconnection");
    }
}
