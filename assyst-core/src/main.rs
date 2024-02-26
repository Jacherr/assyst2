#![feature(let_chains, str_split_whitespace_remainder, round_char_boundary)]

use std::sync::Arc;
use std::time::Duration;

use crate::assyst::{Assyst, ThreadSafeAssyst};
use crate::task::tasks::get_patrons::get_patrons;
use crate::task::tasks::top_gg_stats::post_top_gg_stats;
use crate::task::Task;
use assyst_common::config::config::LoggingWebhook;
use assyst_common::config::CONFIG;
use assyst_common::pipe::{Pipe, GATEWAY_PIPE_PATH};
use assyst_common::util::tracing_init;
use assyst_common::{err, ok_or_break};
use gateway_handler::handle_raw_event;
use gateway_handler::incoming_event::IncomingEvent;
use tokio::spawn;
use tracing::{info, trace};
use twilight_gateway::EventTypeFlags;
use twilight_model::id::marker::WebhookMarker;
use twilight_model::id::Id;

mod assyst;
mod cache_handler;
mod command;
mod downloader;
mod gateway_handler;
mod replies;
mod rest;
mod task;
mod wsi_handler;

// Jemallocator is probably unnecessary for the average instance,
// but when handling hundreds of events per second the performance improvement
// can be measurable
#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[tokio::main]
async fn main() {
    if std::env::consts::OS != "linux" {
        panic!("Assyst is supported on Linux only.")
    }

    tracing_init();

    let assyst: ThreadSafeAssyst = Arc::new(Assyst::new().await.unwrap());

    // Custom panic hook that will send errors to a discord channel
    {
        let handle = tokio::runtime::Handle::current();
        let assyst = Arc::clone(&assyst);

        std::panic::set_hook(Box::new(move |info| {
            println!("{}", info);

            let assyst = assyst.clone();
            let msg = format!("A thread has panicked: ```{}```", info);

            let LoggingWebhook { id, token } = CONFIG.logging_webhooks.panic.clone();

            handle.spawn(async move {
                if id == 0 {
                    err!("Failed to trigger panic webhook: Panic webhook ID is 0");
                } else {
                    let webhook = assyst
                        .http_client
                        .execute_webhook(Id::<WebhookMarker>::new(id), &token)
                        .content(&msg);

                    if let Ok(w) = webhook {
                        let _ = w
                            .await
                            .inspect_err(|e| err!("Failed to trigger panic webhook: {}", e.to_string()));
                    } else if let Err(e) = webhook {
                        err!("Failed to trigger panic webhook: {}", e.to_string());
                    }
                }
            });
        }));
    }

    assyst
        .register_task(Task::new(
            assyst.clone(),
            // 10 mins
            Duration::from_secs(60 * 10),
            function_task_callback!(get_patrons),
        ))
        .await;
    info!("Registered patreon synchronisation task");

    if !CONFIG.dev.disable_bot_list_posting {
        assyst
            .register_task(Task::new(
                assyst.clone(),
                // 10 mins
                Duration::from_secs(60 * 10),
                function_task_callback!(post_top_gg_stats),
            ))
            .await;
        info!("Registered top.gg stats POSTing task");
    } else {
        info!("Bot list POSTing disabled in config: not registering task");
    }

    info!("Starting assyst-webserver");
    assyst_webserver::run(
        assyst.database_handler.clone(),
        assyst.http_client.clone(),
        assyst.prometheus.clone(),
    )
    .await;

    info!("Connecting to assyst-gateway pipe at {}", GATEWAY_PIPE_PATH);
    loop {
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
                    assyst.prometheus.add_event();
                    let assyst_c = assyst.clone();
                    spawn(async move { handle_raw_event(assyst_c.clone(), incoming_event).await });
                }
            }
        }

        err!("Connection to assyst-gateway lost, attempting reconnection");
    }
}

#[cfg(test)]
mod tests {

    use assyst_common::BOT_ID;

    use self::gateway_handler::message_parser::preprocess::message_mention_prefix;

    use super::*;

    #[test]
    fn message_mention_prefix_nick() {
        let prefix_search = format!("<@!{}>", BOT_ID);
        let prefix = message_mention_prefix(&format!("<@!{}> test", BOT_ID));
        assert_eq!(prefix, Some(prefix_search));
    }

    #[test]
    fn message_mention_prefix_no_nick() {
        let prefix_search = format!("<@{}>", BOT_ID);
        let prefix = message_mention_prefix(&format!("<@{}> test", BOT_ID));
        assert_eq!(prefix, Some(prefix_search));
    }

    #[test]
    fn message_mention_prefix_invalid() {
        let prefix = message_mention_prefix(&format!("<{}> test", BOT_ID));
        assert_eq!(prefix, None);
    }
}
