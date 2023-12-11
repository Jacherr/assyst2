use twilight_model::gateway::{
    event::{GatewayEvent, DispatchEvent},
    payload::incoming::{
        GuildCreate, GuildDelete, MessageCreate, MessageDelete, MessageUpdate, Ready,
    },
};



#[derive(Debug)]
pub enum IncomingEvent {
    MessageCreate(MessageCreate),
    MessageUpdate(MessageUpdate),
    MessageDelete(MessageDelete),
    GuildCreate(GuildCreate),
    GuildDelete(GuildDelete),
    ShardReady(Ready),
}
impl TryFrom<GatewayEvent> for IncomingEvent {
    type Error = ();

    fn try_from(event: GatewayEvent) -> Result<Self, ()> {
        match event {
            GatewayEvent::Dispatch(_, event) => {
                match event {
                    DispatchEvent::MessageCreate(message) => Ok(IncomingEvent::MessageCreate(*message)),
                    DispatchEvent::MessageUpdate(message) => Ok(IncomingEvent::MessageUpdate(*message)),
                    DispatchEvent::MessageDelete(message) => Ok(IncomingEvent::MessageDelete(message)),
                    DispatchEvent::GuildCreate(guild) => Ok(IncomingEvent::GuildCreate(*guild)),
                    DispatchEvent::GuildDelete(guild) => Ok(IncomingEvent::GuildDelete(guild)),
                    DispatchEvent::Ready(ready) => Ok(IncomingEvent::ShardReady(*ready)),
                    _ => Err(()),
                }
            },
            _ => Err(())
        }
    }
}