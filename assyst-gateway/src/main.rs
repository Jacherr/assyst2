use std::sync::Arc;

use assyst_common::{config::CONFIG, pipe::Pipe};
use tokio::sync::Mutex;
use tracing::info;
use twilight_gateway::{
    stream::create_recommended,
    Config as GatewayConfig, Intents,
};
use twilight_http::Client as HttpClient;
use twilight_model::gateway::{
    payload::outgoing::update_presence::UpdatePresencePayload,
    presence::{Activity, ActivityType, Status},
};

use crate::gateway_state::GatewayState;

pub mod gateway_state;
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

    info!("Calculating recommended number of shards");

    let mut shards = create_recommended(&http_client, gateway_config.clone(), |_, _| {
        gateway_config.clone()
    })
    .await
    .unwrap()
    .collect::<Vec<_>>();

    info!("Spawning {} shard(s)", shards.len());

    let state = Arc::new(Mutex::new(GatewayState::new(
        Pipe::connect("/tmp/unknown".to_owned()).await.unwrap(),
        http_client,
        shards,
    )));
}
