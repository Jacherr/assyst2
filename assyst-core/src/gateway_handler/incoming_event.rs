use twilight_model::gateway::event::{DispatchEvent, GatewayEvent};
use twilight_model::gateway::payload::incoming::{
    ChannelUpdate, GuildCreate, GuildDelete, GuildUpdate, MessageCreate, MessageDelete, MessageUpdate, Ready,
};

#[derive(Debug)]
pub enum IncomingEvent {
    ChannelUpdate(ChannelUpdate),
    GuildCreate(Box<GuildCreate>), // this struct is huge.
    GuildDelete(GuildDelete),
    GuildUpdate(GuildUpdate),
    MessageCreate(Box<MessageCreate>), // same problem
    MessageDelete(MessageDelete),
    MessageUpdate(MessageUpdate),
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
                DispatchEvent::GuildUpdate(guild) => Ok(IncomingEvent::GuildUpdate(*guild)),
                DispatchEvent::Ready(ready) => Ok(IncomingEvent::ShardReady(*ready)),
                DispatchEvent::ChannelUpdate(channel) => Ok(IncomingEvent::ChannelUpdate(*channel)),
                _ => Err(()),
            },
            _ => Err(()),
        }
    }
}
