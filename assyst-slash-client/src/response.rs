use std::fmt::Display;

use twilight_model::application::command::CommandOptionChoice;
use twilight_model::channel::message::{AllowedMentions, Embed, MessageFlags};
use twilight_model::http::attachment::Attachment;
use twilight_model::http::interaction::{InteractionResponse, InteractionResponseType};

pub struct ResponseBuilder(InteractionResponse);

impl ResponseBuilder {
    #[must_use]
    pub fn new(kind: InteractionResponseType) -> Self {
        Self(InteractionResponse { kind, data: None })
    }

    #[must_use]
    pub fn pong() -> Self {
        Self::new(InteractionResponseType::Pong)
    }

    #[must_use]
    pub fn channel_message_with_source() -> Self {
        Self::new(InteractionResponseType::ChannelMessageWithSource)
    }

    #[must_use]
    pub fn deferred_channel_message_with_source() -> Self {
        Self::new(InteractionResponseType::DeferredChannelMessageWithSource)
    }

    #[must_use]
    pub fn deferred_update_message() -> Self {
        Self::new(InteractionResponseType::DeferredUpdateMessage)
    }

    #[must_use]
    pub fn update_message() -> Self {
        Self::new(InteractionResponseType::UpdateMessage)
    }

    #[must_use]
    pub fn application_command_autocomplete_result() -> Self {
        Self::new(InteractionResponseType::ApplicationCommandAutocompleteResult)
    }

    #[must_use]
    pub fn modal() -> Self {
        Self::new(InteractionResponseType::Modal)
    }

    #[must_use]
    pub fn build(self) -> InteractionResponse {
        self.0
    }

    #[must_use]
    pub fn allowed_mentions(mut self, allowed_mentions: Option<AllowedMentions>) -> Self {
        self.0.data.get_or_insert(<_>::default()).allowed_mentions = allowed_mentions;
        self
    }

    #[must_use]
    pub fn attachments(mut self, attachments: Vec<Attachment>) -> Self {
        self.0.data.get_or_insert(<_>::default()).attachments = Some(attachments);
        self
    }

    #[must_use]
    pub fn choices(mut self, choices: Vec<CommandOptionChoice>) -> Self {
        self.0.data.get_or_insert(<_>::default()).choices = Some(choices);
        self
    }

    #[must_use]
    pub fn content(mut self, content: impl Display) -> Self {
        self.0.data.get_or_insert(<_>::default()).content = Some(format!("{content}"));
        self
    }

    #[must_use]
    pub fn custom_id(mut self, custom_id: impl Display) -> Self {
        self.0.data.get_or_insert(<_>::default()).custom_id = Some(format!("{custom_id}"));
        self
    }

    #[must_use]
    pub fn embeds(mut self, embeds: Vec<Embed>) -> Self {
        self.0.data.get_or_insert(<_>::default()).embeds = Some(embeds);
        self
    }

    #[must_use]
    pub fn suppress_embeds(mut self, value: bool) -> Self {
        let f = self
            .0
            .data
            .get_or_insert(<_>::default())
            .flags
            .get_or_insert(MessageFlags::empty());

        if value {
            f.insert(MessageFlags::SUPPRESS_EMBEDS);
        } else {
            f.remove(MessageFlags::SUPPRESS_EMBEDS);
        }

        self
    }

    #[must_use]
    pub fn ephemeral(mut self, value: bool) -> Self {
        let f = self
            .0
            .data
            .get_or_insert(<_>::default())
            .flags
            .get_or_insert(MessageFlags::empty());

        if value {
            f.insert(MessageFlags::EPHEMERAL);
        } else {
            f.remove(MessageFlags::EPHEMERAL);
        }

        self
    }

    #[must_use]
    pub fn title(mut self, title: impl Display) -> Self {
        self.0.data.get_or_insert(<_>::default()).title = Some(format!("{title}"));
        self
    }

    #[must_use]
    pub fn tts(mut self, tts: bool) -> Self {
        self.0.data.get_or_insert(<_>::default()).tts = Some(tts);
        self
    }
}
