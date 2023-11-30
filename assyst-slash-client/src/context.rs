use std::sync::Arc;

use anyhow::Error;
use twilight_http::{client::InteractionClient, Client};
use twilight_model::{
    application::interaction::{application_command::CommandData, Interaction},
    id::{marker::ApplicationMarker, Id},
};

use crate::response::ResponseBuilder;

#[derive(Clone, Debug)]
pub struct Context<T> {
    pub client: Arc<Client>,
    pub application_id: Id<ApplicationMarker>,
    pub data: T,
}

impl<T> Context<T> {
    pub fn new(client: Arc<Client>, application_id: Id<ApplicationMarker>, custom: T) -> Self {
        Self {
            data: custom,
            client,
            application_id,
        }
    }

    pub fn interactions(&self) -> InteractionClient {
        self.client.interaction(self.application_id)
    }
}

pub struct InnerContext<T> {
    pub ctx: Box<Context<T>>,
    pub interaction: Box<Interaction>,
    pub data: Box<CommandData>,
}

impl<T> InnerContext<T> {
    #[must_use]
    pub fn new(
        ctx: Box<Context<T>>,
        interaction: Box<Interaction>,
        data: Box<CommandData>,
    ) -> Self {
        Self {
            ctx,
            interaction,
            data,
        }
    }

    pub async fn respond(&self, data: ResponseBuilder) -> Result<(), Error> {
        self.ctx
            .interactions()
            .create_response(self.interaction.id, &self.interaction.token, &data.build())
            .await?;
        Ok(())
    }
}
