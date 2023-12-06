use self::incoming_event::IncomingEvent;
use tracing::info;

pub mod incoming_event;

pub fn handle_raw_event(event: IncomingEvent) {
    match event {
        IncomingEvent::ShardReady(ready) => {
            if let Some(shard) = ready.shard {
                info!("Shard {} of {}: READY in {} guilds", shard.number(), shard.total() - 1, ready.guilds.len())
            }
        },
        IncomingEvent::MessageCreate(_) => {
            //println!("message create");
        },
        IncomingEvent::MessageUpdate(_) => {
            //println!("message update");
        },
        IncomingEvent::MessageDelete(_) => {
            //println!("message delete");
        },
        IncomingEvent::GuildCreate(_) => {
            //println!("guild create");
        },
        IncomingEvent::GuildDelete(_) => {
            //println!("guild delete");
        }
    }
}