use std::sync::Arc;

use assyst_common::config::CONFIG;
use futures_util::stream::StreamExt;
use parser::{handle_raw_event, incoming_event::IncomingEvent};
use tokio::sync::Mutex;
use tracing::info;
use twilight_gateway::{
    stream::{create_recommended, ShardMessageStream},
    Config as GatewayConfig, EventTypeFlags, Intents, Message,
};
use twilight_http::Client as HttpClient;
use twilight_model::gateway::{
    payload::outgoing::update_presence::UpdatePresencePayload,
    presence::{Activity, ActivityType, Status},
};

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

    let http_client = HttpClient::new(CONFIG.authentication.discord_token.clone());

    let presence =
        UpdatePresencePayload::new(vec![ACTIVITY.to_owned()], false, None, Status::Online).unwrap();

    let gateway_config = GatewayConfig::builder(
        CONFIG.authentication.discord_token.clone(),
        Intents::MESSAGE_CONTENT | Intents::GUILDS | Intents::GUILD_MESSAGES,
    )
    .presence(presence)
    .build();

    info!("Calculating recommended number of shards...");

    let mut shards = create_recommended(&http_client, gateway_config.clone(), |_, _| {
        gateway_config.clone()
    })
    .await
    .unwrap()
    .collect::<Vec<_>>();

    info!("Spawning {} shard(s)", shards.len());

    let stream = Arc::new(Mutex::new(ShardMessageStream::new(shards.iter_mut())));

    while let Some((_, event)) = stream.lock().await.next().await {
        if let Ok(Message::Text(event)) = event {
            let parsed_event = twilight_gateway::parse(
                event,
                EventTypeFlags::GUILD_CREATE
                    | EventTypeFlags::GUILD_DELETE
                    | EventTypeFlags::MESSAGE_CREATE
                    | EventTypeFlags::MESSAGE_DELETE
                    | EventTypeFlags::MESSAGE_UPDATE
                    | EventTypeFlags::READY,
            ).ok().flatten();

            if let Some(parsed_event) = parsed_event {
                let try_incoming_event: Result<IncomingEvent, _> = parsed_event.try_into();
                if let Ok(incoming_event) = try_incoming_event {
                    tokio::spawn(async {
                        handle_raw_event(incoming_event);
                    });
                }
            }
        }
    }
}
