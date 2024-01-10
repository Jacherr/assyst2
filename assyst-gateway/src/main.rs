#![feature(fs_try_exists)]

use assyst_common::config::CONFIG;
use assyst_common::gateway::core_event::CoreEvent;
use assyst_common::ok_or_break;
use assyst_common::pipe::pipe_server::PipeServer;
use assyst_common::pipe::GATEWAY_PIPE_PATH;
use futures_util::StreamExt;
use std::sync::Arc;
use tokio::sync::mpsc::unbounded_channel;
use tokio::sync::Mutex;
use tracing::{debug, info, trace};
use twilight_gateway::stream::{create_recommended, ShardMessageStream};
use twilight_gateway::{Config as GatewayConfig, EventTypeFlags, Intents, Message};
use twilight_http::Client as HttpClient;
use twilight_model::gateway::payload::outgoing::update_presence::UpdatePresencePayload;
use twilight_model::gateway::presence::{Activity, ActivityType, Status};

use crate::parser::handle_raw_event;
use crate::parser::incoming_event::IncomingEvent;

pub mod parser;

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

    if std::fs::try_exists(GATEWAY_PIPE_PATH).is_ok() {
        info!("Deleting old pipe file {}", GATEWAY_PIPE_PATH);
        std::fs::remove_file(GATEWAY_PIPE_PATH).unwrap();
    }

    let http_client = HttpClient::new(CONFIG.authentication.discord_token.clone());

    let presence = UpdatePresencePayload::new(vec![ACTIVITY.to_owned()], false, None, Status::Online).unwrap();

    let intents = Intents::MESSAGE_CONTENT | Intents::GUILDS | Intents::GUILD_MESSAGES;
    debug!("intents={:?}", intents);
    let gateway_config = GatewayConfig::builder(
        CONFIG.authentication.discord_token.clone(),
        intents,
    )
    .presence(presence)
    .build();

    let mut shards = create_recommended(&http_client, gateway_config.clone(), |_, _| gateway_config.clone())
        .await
        .unwrap()
        .collect::<Vec<_>>();

    info!("Recommended shard count: {}", shards.len());

    let mut pipe_server = PipeServer::listen(GATEWAY_PIPE_PATH).unwrap();
    info!(
        "Listening for incoming connections from assyst-core on {}",
        GATEWAY_PIPE_PATH
    );

    // pipe thread
    let (tx, mut rx) = unbounded_channel::<CoreEvent>();
    tokio::spawn(async move {
        loop {
            info!("Awaiting connection from assyst-core");
            if let Ok(mut stream) = pipe_server.accept_connection().await {
                info!("Connection received from assyst-core");
                loop {
                    if let Some(data) = rx.recv().await {
                        debug!("core event received: {:?}", data);
                        ok_or_break!(stream.write_object(data).await);
                    } else {
                        break;
                    };
                }
            }
        }
    });

    let message_stream = Arc::new(Mutex::new(ShardMessageStream::new(shards.iter_mut())));

    while let Some((_, event)) = message_stream.lock().await.next().await {
        if let Ok(Message::Text(event)) = event {
            trace!("discord message received: {}", event);

            let parsed_event = twilight_gateway::parse(
                event,
                EventTypeFlags::GUILD_CREATE
                    | EventTypeFlags::GUILD_DELETE
                    | EventTypeFlags::MESSAGE_CREATE
                    | EventTypeFlags::MESSAGE_DELETE
                    | EventTypeFlags::MESSAGE_UPDATE
                    | EventTypeFlags::READY,
            )
            .ok()
            .flatten();

            if let Some(parsed_event) = parsed_event {
                let try_incoming_event: Result<IncomingEvent, _> = parsed_event.try_into();
                if let Ok(incoming_event) = try_incoming_event {
                    handle_raw_event(incoming_event, tx.clone()).await;
                }
            }
        }
    }
}
