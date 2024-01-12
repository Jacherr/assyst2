use std::sync::Arc;

use anyhow::bail;
use assyst_common::config::CONFIG;
use assyst_common::gateway::core_event::CoreEventSender;
use assyst_common::pipe::pipe_server::PipeServer;
use assyst_common::pipe::{Pipe, GATEWAY_PIPE_PATH};
use assyst_database::DatabaseHandler;
use tokio::sync::Mutex;

pub type ThreadSafeGatewayContext = Arc<Mutex<GatewayContext>>;

/// Mutex-locked context class with key information about the gateway state.
pub struct GatewayContext {
    core_event_sender: Option<CoreEventSender>,
    pub database_handler: DatabaseHandler, /* database connection here
                                            * ... anything else here */
}
impl GatewayContext {
    pub async fn new() -> Self {
        GatewayContext {
            core_event_sender: None,
            database_handler: DatabaseHandler::new(CONFIG.database.to_url()).await.unwrap(),
        }
    }

    pub fn set_core_event_sender(&mut self, sender: CoreEventSender) {
        self.core_event_sender = Some(sender);
    }

    pub fn clone_core_event_sender(&self) -> Option<CoreEventSender> {
        self.core_event_sender.clone()
    }
}
