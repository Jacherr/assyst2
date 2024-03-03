use twilight_model::gateway::event::{DispatchEvent, GatewayEvent};
use twilight_model::gateway::payload::incoming::{
    GuildCreate, GuildDelete, MessageCreate, MessageDelete, MessageUpdate, Ready,
};

#[derive(Debug)]
pub enum IncomingEvent {
    MessageCreate(Box<MessageCreate>), // this struct is huge.
    MessageUpdate(MessageUpdate),
    MessageDelete(MessageDelete),
    GuildCreate(Box<GuildCreate>), // same problem
    GuildDelete(GuildDelete),
    ShardReady(Ready),
}
impl TryFrom<GatewayEvent> for IncomingEvent {
    type Error = ();

    fn try_from(event: GatewayEvent) -> Result<Self, ()> {
        match event {
            GatewayEvent::Dispatch(_, event) => match event {
                DispatchEvent::MessageCreate(message) => Ok(IncomingEvent::MessageCreate(message)),
                DispatchEvent::MessageUpdate(message) => Ok(IncomingEvent::MessageUpdate(*message)),
                DispatchEvent::MessageDelete(message) => Ok(IncomingEvent::MessageDelete(message)),
                DispatchEvent::GuildCreate(guild) => Ok(IncomingEvent::GuildCreate(guild)),
                DispatchEvent::GuildDelete(guild) => Ok(IncomingEvent::GuildDelete(guild)),
                DispatchEvent::Ready(ready) => Ok(IncomingEvent::ShardReady(*ready)),
                _ => Err(()),
            },
            _ => Err(()),
        }
    }
}
