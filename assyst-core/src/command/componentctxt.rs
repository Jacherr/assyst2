use std::sync::Arc;
use std::time::Duration;

use moka::sync::Cache;
use tokio::sync::Mutex;
use twilight_model::application::interaction::message_component::MessageComponentInteractionData;
use twilight_model::application::interaction::modal::ModalInteractionData;
use twilight_model::channel::message::component::{Button, ButtonStyle};
use twilight_model::channel::message::{AllowedMentions, Component, EmojiReactionType, MessageFlags};
use twilight_model::http::interaction::{InteractionResponse, InteractionResponseType};
use twilight_model::id::Id;
use twilight_model::id::marker::{GuildMarker, InteractionMarker, UserMarker};
use twilight_util::builder::InteractionResponseDataBuilder;

use super::misc::tag::TagPaginatorComponentMetadata;
use crate::assyst::ThreadSafeAssyst;

/// A register of all custom IDs that will trigger a certain component context callback.
pub type ComponentCtxtRegister = (Vec<String>, ComponentCtxt);

pub struct ComponentInteractionData {
    pub assyst: ThreadSafeAssyst,
    #[allow(unused)]
    pub message_interaction_data: Option<Box<MessageComponentInteractionData>>,
    pub modal_submit_interaction_data: Option<ModalInteractionData>,
    pub invocation_user_id: Id<UserMarker>,
    pub invocation_guild_id: Option<Id<GuildMarker>>,
    pub interaction_id: Id<InteractionMarker>,
    pub interaction_token: String,
    pub custom_id: String,
}

/// A component context is a context in which a component interaction is handled under.\
/// It contains basic information required to action on the button.\
/// Because components are responded to via interactions, minimal metadata (e.g., from
/// `CommandData`) is required.
#[derive(Clone)]
pub struct ComponentCtxt {
    pub assyst: ThreadSafeAssyst,
    pub data: ComponentMetadata,
}
impl ComponentCtxt {
    pub fn new(assyst: ThreadSafeAssyst, data: ComponentMetadata) -> Self {
        Self { assyst, data }
    }

    pub async fn handle_component_interaction(
        &mut self,
        component_data: &ComponentInteractionData,
    ) -> anyhow::Result<()> {
        // handler functions return new componentctxt which we can then update the cache with, for any
        // further interactions
        let res = match &mut self.data {
            ComponentMetadata::TagList(tl) => tl.component_callback(component_data).await,
        };

        if let Err(e) = res {
            respond_new_invis(
                self.assyst.clone(),
                component_data.interaction_id,
                &component_data.interaction_token,
                &format!(":warning: ``{e:#}``"),
            )
            .await?;
        };

        Ok(())
    }
}

pub async fn respond_new_invis(
    assyst: ThreadSafeAssyst,
    interaction_id: Id<InteractionMarker>,
    interaction_token: &str,
    text: &str,
) -> anyhow::Result<()> {
    let b = InteractionResponseDataBuilder::new();
    let b = b.allowed_mentions(AllowedMentions::default());
    let b = b.content(text);
    let b = b.flags(MessageFlags::EPHEMERAL);
    let r = b.build();
    let r = InteractionResponse {
        kind: InteractionResponseType::ChannelMessageWithSource,
        data: Some(r),
    };
    assyst
        .interaction_client()
        .create_response(interaction_id, interaction_token, &r)
        .await?;

    Ok(())
}

pub async fn respond_update_text(
    assyst: ThreadSafeAssyst,
    interaction_id: Id<InteractionMarker>,
    interaction_token: &str,
    text: &str,
) -> anyhow::Result<()> {
    let b = InteractionResponseDataBuilder::new();
    let b = b.allowed_mentions(AllowedMentions::default());
    let b = b.content(text);
    let r = b.build();
    let r = InteractionResponse {
        kind: InteractionResponseType::UpdateMessage,
        data: Some(r),
    };
    assyst
        .interaction_client()
        .create_response(interaction_id, interaction_token, &r)
        .await?;
    Ok(())
}

pub async fn respond_modal(
    assyst: ThreadSafeAssyst,
    interaction_id: Id<InteractionMarker>,
    interaction_token: &str,
    title: &str,
    components: Vec<Component>,
    custom_id: &str,
) -> anyhow::Result<()> {
    let b = InteractionResponseDataBuilder::new();
    let b = b.custom_id(custom_id);
    let b = b.title(title);
    let b = b.components(components);
    let r = b.build();
    let r = InteractionResponse {
        kind: InteractionResponseType::Modal,
        data: Some(r),
    };
    assyst
        .interaction_client()
        .create_response(interaction_id, interaction_token, &r)
        .await?;
    Ok(())
}

#[derive(Clone)]
pub enum ComponentMetadata {
    TagList(TagPaginatorComponentMetadata),
}

pub fn button_emoji_new(custom_id: &str, emoji: EmojiReactionType, style: ButtonStyle) -> Button {
    Button {
        id: None,
        custom_id: Some(custom_id.to_owned()),
        disabled: false,
        emoji: Some(emoji),
        label: None,
        style,
        url: None,
        sku_id: None,
    }
}

pub fn button_new(custom_id: &str, label: &str, style: ButtonStyle) -> Button {
    Button {
        id: None,
        custom_id: Some(custom_id.to_owned()),
        disabled: false,
        emoji: None,
        label: Some(label.to_owned()),
        style,
        url: None,
        sku_id: None,
    }
}

/// Map of all existing component contexts.
pub struct ComponentCtxts(Cache<String, Arc<Mutex<ComponentCtxt>>>);
impl ComponentCtxts {
    pub fn new() -> Self {
        Self(
            Cache::builder()
                .max_capacity(1000)
                .time_to_live(Duration::from_secs(10 * 60))
                .build(),
        )
    }

    pub fn insert(&self, cid: &str, ctxt: &Arc<Mutex<ComponentCtxt>>) {
        self.0.insert(cid.to_owned(), ctxt.clone());
    }

    pub fn get(&self, cid: &str) -> Option<Arc<Mutex<ComponentCtxt>>> {
        self.0.get(cid)
    }
}
