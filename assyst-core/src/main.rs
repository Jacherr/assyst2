use std::sync::Arc;

use assyst::{Assyst, ThreadSafeAssyst};
use assyst_common::ok_or_break;
use assyst_common::pipe::{Pipe, GATEWAY_PIPE_PATH};
use gateway_handler::handle_raw_event;
use gateway_handler::incoming_event::IncomingEvent;
use tokio::sync::Mutex;
use tracing::{info, trace};
use twilight_gateway::EventTypeFlags;

mod assyst;
mod gateway_handler;

#[tokio::main]
async fn main() {
    info!("Initialising");
    let assyst: ThreadSafeAssyst = Arc::new(Mutex::new(Assyst::new().await.unwrap()));

    loop {
        info!("Connecting to assyst-gateway pipe at {}", GATEWAY_PIPE_PATH);
        let mut gateway_pipe = Pipe::poll_connect(GATEWAY_PIPE_PATH, None).await.unwrap();
        info!("Connected to assyst-gateway pipe at {}", GATEWAY_PIPE_PATH);

        loop {
            // break if read fails because it means broken pipe
            // we need to re-poll the pipe to get a new connection
            let event = ok_or_break!(gateway_pipe.read_string().await);
            trace!("got event: {}", event);

            let parsed_event = twilight_gateway::parse(
                event,
                EventTypeFlags::GUILD_CREATE
                    | EventTypeFlags::GUILD_DELETE
                    | EventTypeFlags::MESSAGE_CREATE
                    | EventTypeFlags::MESSAGE_DELETE
                    | EventTypeFlags::MESSAGE_UPDATE
                    | EventTypeFlags::READY,
            )
            .ok()
            .flatten();

            if let Some(parsed_event) = parsed_event {
                let try_incoming_event: Result<IncomingEvent, _> = parsed_event.try_into();
                if let Ok(incoming_event) = try_incoming_event {
                    handle_raw_event(assyst.clone(), incoming_event).await;
                }
            }
        }
    }
}
