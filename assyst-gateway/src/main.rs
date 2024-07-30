use assyst_common::config::CONFIG;
use assyst_common::ok_or_break;
use assyst_common::pipe::pipe_server::PipeServer;
use assyst_common::pipe::GATEWAY_PIPE_PATH;
use assyst_common::util::tracing_init;
use futures_util::StreamExt;
use tokio::sync::mpsc::{channel, Sender};
use tokio::{signal, spawn};
use tracing::{debug, info, trace, warn};
use twilight_gateway::{create_recommended, ConfigBuilder as GatewayConfigBuilder, Intents, Message, Shard};
use twilight_http::Client as HttpClient;
use twilight_model::gateway::payload::outgoing::update_presence::UpdatePresencePayload;
use twilight_model::gateway::presence::{Activity, ActivityType, Status};

// Jemallocator is probably unnecessary for the average instance,
// but when handling hundreds of events per second the performance improvement
// can be measurable
#[cfg(target_os = "linux")]
#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

lazy_static::lazy_static! {
    static ref ACTIVITY: Activity = Activity {
        application_id: None,
        assets: None,
        created_at: None,
        details: None,
        emoji: None,
        flags: None,
        id: None,
        instance: None,
        kind: ActivityType::Playing,
        name: format!("{}help | jacher.io/assyst", CONFIG.prefix.default),
        party: None,
        secrets: None,
        state: None,
        timestamps: None,
        url: None,
        buttons: Vec::new(),
    };
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if std::env::consts::OS != "linux" {
        panic!("Assyst is supported on Linux only.")
    }

    tracing_init();

    let http_client = HttpClient::new(CONFIG.authentication.discord_token.clone());

    let presence = UpdatePresencePayload::new(vec![ACTIVITY.to_owned()], false, None, Status::Online).unwrap();

    let intents = Intents::MESSAGE_CONTENT | Intents::GUILDS | Intents::GUILD_MESSAGES | Intents::DIRECT_MESSAGES;
    debug!("intents={:?}", intents);
    let gateway_config = GatewayConfigBuilder::new(CONFIG.authentication.discord_token.clone(), intents)
        .presence(presence)
        .build();

    let shards = create_recommended(&http_client, gateway_config.clone(), |_, _| gateway_config.clone())
        .await
        .unwrap()
        .collect::<Vec<_>>();

    info!("Recommended shard count: {}", shards.len());

    // pipe thread tx/rx
    let (tx, mut rx) = channel::<String>(10);

    let mut core_pipe_server = PipeServer::listen(GATEWAY_PIPE_PATH).unwrap();
    info!("Core listener started on {}", GATEWAY_PIPE_PATH);

    // pipe thread
    tokio::spawn(async move {
        info!("Awaiting connection from assyst-core");
        loop {
            if let Ok(mut stream) = core_pipe_server.accept_connection().await {
                info!("Connection received from assyst-core");
                while let Some(v) = rx.recv().await {
                    ok_or_break!(stream.write_string(v).await);
                }
                warn!("Connection to assyst-core lost, awaiting reconnection");
            }
        }
    });

    let mut tasks = vec![];
    let shards_count = shards.len();

    for shard in shards {
        info!(
            "Registering runner for shard {} of {}",
            shard.id().number(),
            shards_count - 1
        );
        tasks.push(spawn(runner(shard, tx.clone())));
    }

    signal::ctrl_c().await?;

    Ok(())
}

async fn runner(mut shard: Shard, tx: Sender<String>) {
    loop {
        match shard.next().await {
            Some(Ok(Message::Text(message))) => {
                trace!("got message: {message}");
                let _ = tx.try_send(message);
            },
            Some(Err(e)) => {
                warn!(?e, "error receiving event");
            },
            _ => {},
        }
    }
}
