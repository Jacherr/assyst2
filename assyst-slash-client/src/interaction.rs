use std::{any::Any, sync::Arc};

use twilight_gateway::Event;
use twilight_http::client::InteractionClient;
use twilight_model::{
    application::interaction::{
        application_command::CommandData, InteractionData, InteractionType,
    },
    gateway::payload::incoming::InteractionCreate,
    http::interaction::{InteractionResponse, InteractionResponseData, InteractionResponseType},
};

#[allow(clippy::needless_lifetimes)]
pub async fn process<'a>(event: Event, client: Arc<InteractionClient<'a>>) -> anyhow::Result<()> {
    let mut i = match event {
        Event::InteractionCreate(i) => i,
        _ => return Ok(()),
    };

    let d = match std::mem::take(&mut i.data) {
        // assuming we don't press any buttons from anywhere just yet
        Some(InteractionData::ApplicationCommand(d)) => d,
        _ => return Ok(()),
    };

    match d.name.as_str() {
        "ping" => ping(client, i, d).await?,
        "unreachable" => unsafe { std::hint::unreachable_unchecked() }, // shut the fuck up clippy
        // not a command we care about
        _ => (),
    };

    Ok(())
}

#[allow(clippy::needless_lifetimes)]
pub async fn ping<'a>(
    client: Arc<InteractionClient<'a>>,
    interaction: Box<InteractionCreate>,
    data: Box<CommandData>,
) -> anyhow::Result<()> {
    client
        .create_response(
            interaction.id,
            &interaction.token,
            &InteractionResponse {
                kind: InteractionResponseType::ChannelMessageWithSource,
                data: Some(InteractionResponseData {
                    content: Some(String::from("pong")),
                    ..<_>::default()
                }),
            },
        )
        .await?;

    Ok(())
}
