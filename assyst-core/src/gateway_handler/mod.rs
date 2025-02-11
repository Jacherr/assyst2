#![allow(clippy::match_single_binding, clippy::single_match)] // shh...
use self::incoming_event::IncomingEvent;
use crate::assyst::ThreadSafeAssyst;

pub mod event_handlers;
pub mod incoming_event;
pub mod message_parser;
pub mod reply;

/// Checks the enum variant of this `IncomingEvent` and calls the appropriate handler function
/// for further processing.
pub async fn handle_raw_event(context: ThreadSafeAssyst, event: IncomingEvent) {
    match event {
        IncomingEvent::ShardReady(event) => {
            event_handlers::ready::handle(context, event).await;
        },
        IncomingEvent::MessageCreate(event) => {
            event_handlers::message_create::handle(context, *event).await;
        },
        IncomingEvent::MessageUpdate(event) => {
            event_handlers::message_update::handle(context, event).await;
        },
        IncomingEvent::MessageDelete(event) => {
            event_handlers::message_delete::handle(context, event).await;
        },
        IncomingEvent::GuildCreate(event) => {
            event_handlers::guild_create::handle(context, *event).await;
        },
        IncomingEvent::GuildDelete(event) => {
            event_handlers::guild_delete::handle(context, event).await;
        },
        IncomingEvent::GuildUpdate(event) => {
            event_handlers::guild_update::handle(context, event);
        },
        IncomingEvent::ChannelUpdate(event) => {
            event_handlers::channel_update::handle(context, event);
        },
        IncomingEvent::InteractionCreate(event) => {
            event_handlers::interaction_create::handle(context, *event).await;
        },
        IncomingEvent::EntitlementCreate(event) => {
            event_handlers::entitlement_create::handle(context, event).await;
        },
        IncomingEvent::EntitlementUpdate(event) => {
            event_handlers::entitlement_update::handle(context, event).await;
        },
        IncomingEvent::EntitlementDelete(event) => {
            event_handlers::entitlement_delete::handle(context, event).await;
        },
    }
}
