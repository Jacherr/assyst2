#![feature(fs_try_exists)]

use assyst_common::config::CONFIG;
use assyst_common::ok_or_break;
use assyst_common::pipe::pipe_server::PipeServer;
use assyst_common::pipe::GATEWAY_PIPE_PATH;
use futures_util::StreamExt;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::mpsc::unbounded_channel;
use tokio::sync::Mutex;
use tracing::{debug, info, trace};
use twilight_gateway::stream::{create_recommended, ShardMessageStream};
use twilight_gateway::{Config as GatewayConfig, Intents, Message};
use twilight_http::Client as HttpClient;
use twilight_model::gateway::payload::outgoing::update_presence::UpdatePresencePayload;
use twilight_model::gateway::presence::{Activity, ActivityType, Status};

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
async fn main() {
    tracing_subscriber::fmt::init();

    info!("Initialising");

    if Path::new(GATEWAY_PIPE_PATH).exists() {
        info!("Deleting old pipe file {}", GATEWAY_PIPE_PATH);
        std::fs::remove_file(GATEWAY_PIPE_PATH).unwrap();
    }

    let http_client = HttpClient::new(CONFIG.authentication.discord_token.clone());

    let presence = UpdatePresencePayload::new(vec![ACTIVITY.to_owned()], false, None, Status::Online).unwrap();

    let intents = Intents::MESSAGE_CONTENT | Intents::GUILDS | Intents::GUILD_MESSAGES;
    debug!("intents={:?}", intents);
    let gateway_config = GatewayConfig::builder(CONFIG.authentication.discord_token.clone(), intents)
        .presence(presence)
        .build();

    let mut shards = create_recommended(&http_client, gateway_config.clone(), |_, _| gateway_config.clone())
        .await
        .unwrap()
        .collect::<Vec<_>>();

    info!("Recommended shard count: {}", shards.len());

    // pipe thread tx/rx
    let (tx, mut rx) = unbounded_channel::<String>();

    let mut core_pipe_server = PipeServer::listen(GATEWAY_PIPE_PATH).unwrap();
    info!("Core listener started on {}", GATEWAY_PIPE_PATH);

    // pipe thread
    tokio::spawn(async move {
        loop {
            info!("Awaiting connection from assyst-core");
            if let Ok(mut stream) = core_pipe_server.accept_connection().await {
                info!("Connection received from assyst-core");
                loop {
                    let data = ok_or_break!(rx.recv().await.ok_or(()));
                    ok_or_break!(stream.write_string(data).await);
                }
            }
        }
    });

    let message_stream = Arc::new(Mutex::new(ShardMessageStream::new(shards.iter_mut())));

    while let Some((_, event)) = message_stream.lock().await.next().await {
        if let Ok(Message::Text(event)) = event {
            trace!("discord message received: {}", event);

            tx.send(event).unwrap();
        }
    }
}
