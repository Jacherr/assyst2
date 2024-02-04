use assyst_common::cache::CacheJob;
use assyst_common::ok_or_break;
use assyst_common::pipe::pipe_server::PipeServer;
use assyst_common::pipe::CACHE_PIPE_PATH;
use assyst_common::util::tracing_init;
use tracing::{info, warn};

#[tokio::main]
async fn main() {
    tracing_init();

    let mut pipe_server = PipeServer::listen(CACHE_PIPE_PATH).unwrap();
    info!("Awaiting connection from assyst-core");
    loop {
        let mut stream = pipe_server.accept_connection().await.unwrap();
        info!("Connection received from assyst-core");
        loop {
            let job = ok_or_break!(stream.read_object::<CacheJob>().await);
            // do stuff with job...
        }
        warn!("Connection to assyst-core lost, awaiting reconnection");
    }
}
