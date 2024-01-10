use self::incoming_event::IncomingEvent;
use assyst_common::gateway::core_event::CoreEventSender;

pub mod incoming_event;
pub mod event_handlers;

/// Checks the enum variant of this IncomingEvent and calls the appropriate handler function
/// for further processing.
pub async fn handle_raw_event(event: IncomingEvent, tx: CoreEventSender) {
    match event {
        IncomingEvent::ShardReady(ready) => {
            event_handlers::ready::handle(ready);
        },
        IncomingEvent::MessageCreate(event) => {
            event_handlers::message_create::handle(event, tx).await;
        },
        IncomingEvent::MessageUpdate(event) => {
            event_handlers::message_update::handle(event, tx).await;
        },
        IncomingEvent::MessageDelete(event) => {
            event_handlers::message_delete::handle(event);
        },
        IncomingEvent::GuildCreate(event) => {
            event_handlers::guild_create::handle(event);
        },
        IncomingEvent::GuildDelete(event) => {
            event_handlers::guild_delete::handle(event);
        }
    }
}