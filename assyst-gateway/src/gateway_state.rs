use std::sync::Arc;

use assyst_common::pipe::Pipe;
use futures_util::StreamExt;
use tokio::sync::Mutex;
use twilight_gateway::{stream::ShardMessageStream, EventTypeFlags, Message, Shard};
use twilight_http::Client as HttpClient;

use crate::parser::{handle_raw_event, incoming_event::IncomingEvent};

pub struct GatewayState<'a> {
    pub assyst_connection: Pipe,
    pub http_client: HttpClient,
    pub shards: Vec<Shard>,
    pub message_stream: Option<Arc<Mutex<ShardMessageStream<'a>>>>
}
impl<'a> GatewayState<'a> {
    pub fn new(
        assyst_connection: Pipe,
        http_client: HttpClient,
        shards: Vec<Shard>,
    ) -> GatewayState<'a> {
        GatewayState {
            assyst_connection,
            http_client,
            shards,
            message_stream: None
        }
    }

    pub async fn init_event_stream(&mut self) {
        self.message_stream = Some(Arc::new(Mutex::new(ShardMessageStream::new(self.shards.iter_mut()))));
        let shard_stream = self.message_stream.unwrap().clone().as_ref();

        tokio::spawn(async move {
            while let Some((_, event)) = shard_stream.lock().await.next().await {
                if let Ok(Message::Text(event)) = event {
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
                            tokio::spawn(async {
                                handle_raw_event(incoming_event);
                            });
                        }
                    }
                }
            }
        });
    }
}
