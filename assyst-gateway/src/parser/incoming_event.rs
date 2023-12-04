use twilight_model::gateway::{payload::incoming::{MessageCreate, MessageUpdate, MessageDelete, GuildCreate, GuildDelete}, event::{GatewayEvent, DispatchEvent}};

pub enum IncomingEvent {
    MessageCreate(MessageCreate),
    MessageUpdate(MessageUpdate),
    MessageDelete(MessageDelete),
    GuildCreate(GuildCreate),
    GuildDelete(GuildDelete)
}
impl TryFrom<GatewayEvent> for IncomingEvent {
    type Error = ();

    fn try_from(value: GatewayEvent) -> Result<Self, ()> {
        match value {
            GatewayEvent::Dispatch(_, event) => {
                match event {
                    DispatchEvent::MessageCreate(message) => Ok(IncomingEvent::MessageCreate(*message)),
                    DispatchEvent::MessageUpdate(message) => Ok(IncomingEvent::MessageUpdate(*message)),
                    DispatchEvent::MessageDelete(message) => Ok(IncomingEvent::MessageDelete(message)),
                    DispatchEvent::GuildCreate(guild) => Ok(IncomingEvent::GuildCreate(*guild)),
                    DispatchEvent::GuildDelete(guild) => Ok(IncomingEvent::GuildDelete(guild)),
                    _ => Err(())
                }
            },
            _ => Err(())
        }
    }
}