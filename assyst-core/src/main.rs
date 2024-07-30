#![feature(
    let_chains,
    str_split_whitespace_remainder,
    round_char_boundary,
    trait_alias,
    async_closure,
    if_let_guard,
    iterator_try_collect
)]

use std::sync::Arc;
use std::time::Duration;

use assyst_common::config::config::LoggingWebhook;
use assyst_common::config::CONFIG;
use assyst_common::pipe::{Pipe, GATEWAY_PIPE_PATH};
use assyst_common::util::tracing_init;
use assyst_common::{err, ok_or_break};
use command::registry::register_interaction_commands;
use flux_handler::FluxHandler;
use gateway_handler::handle_raw_event;
use gateway_handler::incoming_event::IncomingEvent;
use rest::web_media_download::get_web_download_api_urls;
use task::tasks::refresh_web_download_urls::refresh_web_download_urls;
use task::tasks::reminders::handle_reminders;
use tokio::spawn;
use tracing::{debug, info /* trace */};
use twilight_gateway::EventTypeFlags;
use twilight_model::id::marker::WebhookMarker;
use twilight_model::id::Id;

use crate::assyst::{Assyst, ThreadSafeAssyst};
use crate::task::tasks::get_premium_users::get_premium_users;
use crate::task::tasks::top_gg_stats::post_top_gg_stats;
use crate::task::Task;

mod assyst;
mod bad_translator;
mod command;
mod command_ratelimits;
mod downloader;
mod flux_handler;
mod gateway_handler;
mod persistent_cache_handler;
mod replies;
mod rest;
mod task;

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
            println!("{info}");

            let assyst = assyst.clone();
            let msg = format!("A thread has panicked: ```{info}```");

            if CONFIG.logging_webhooks.enable_webhooks {
                let LoggingWebhook { id, token } = CONFIG.logging_webhooks.panic.clone();

                handle.spawn(async move {
                    if id == 0 {
                        err!("Failed to trigger panic webhook: Panic webhook ID is 0");
                    } else {
                        let _ = assyst
                            .http_client
                            .execute_webhook(Id::<WebhookMarker>::new(id), &token)
                            .content(&msg)
                            .await
                            .inspect_err(|e| err!("Failed to trigger panic webhook: {}", e.to_string()));
                    }
                });
            }
        }));
    }

    if CONFIG.dev.disable_patreon_synchronisation {
        info!(
            "Patreon synchronisation disabled in config.dev.disable_patreson_synchronisation, will only load admins as patrons"
        );
    }

    assyst.register_task(Task::new(
        assyst.clone(),
        // 10 mins
        Duration::from_secs(60 * 10),
        function_task_callback!(get_premium_users),
    ));
    info!("Registered patreon synchronisation task");

    if !CONFIG.dev.disable_bot_list_posting {
        assyst.register_task(Task::new_delayed(
            assyst.clone(),
            // 10 mins
            Duration::from_secs(60 * 10),
            Duration::from_secs(60 * 10),
            function_task_callback!(post_top_gg_stats),
        ));
        info!("Registered top.gg stats POSTing task");
    } else {
        info!("Bot list POSTing disabled in config.dev.disable_bot_list_posting: not registering task");
    }

    if !CONFIG.dev.disable_reminder_check {
        assyst.register_task(Task::new(
            assyst.clone(),
            Duration::from_millis(crate::task::tasks::reminders::FETCH_INTERVAL as u64),
            function_task_callback!(handle_reminders),
        ));
        info!("Registered reminder check task");
    } else {
        info!("Reminder processing disabled in config.dev.disable_reminder_check: not registering task");
    }

    assyst.register_task(Task::new_delayed(
        assyst.clone(),
        Duration::from_secs(60 * 10),
        Duration::from_secs(60 * 10),
        function_task_callback!(refresh_web_download_urls),
    ));
    info!("Registered web download url refreshing task");

    info!("Starting assyst-webserver");
    assyst_webserver::run(
        assyst.database_handler.clone(),
        assyst.http_client.clone(),
        assyst.metrics_handler.clone(),
    )
    .await;

    info!("Registering interaction commands");
    register_interaction_commands(assyst.clone()).await.unwrap();

    assyst
        .metrics_handler
        .add_guilds(assyst.persistent_cache_handler.get_guild_count().await.unwrap_or(0));

    if !CONFIG.dev.disable_bad_translator_channels {
        info!("Initialising BadTranslator channels");
        assyst.init_badtranslator_channels().await;
    } else {
        info!("BadTranslator channels disabled in config.dev.disable_bad_translator_channels, skipping init");
    }

    let a = assyst.clone();
    spawn(async move {
        info!("Compiling Flux...");
        if let Err(e) = FluxHandler::compile_flux().await {
            err!("Failed to compile flux: {e}");
        } else {
            info!(
                "Flux is compiled (version: {})",
                a.flux_handler.get_version().await.unwrap().trim()
            );
        }
    });

    let a = assyst.clone();

    spawn(async move {
        info!("Connecting to assyst-gateway pipe at {}", GATEWAY_PIPE_PATH);
        loop {
            let mut gateway_pipe = Pipe::poll_connect(GATEWAY_PIPE_PATH, None).await.unwrap();
            info!("Connected to assyst-gateway pipe at {}", GATEWAY_PIPE_PATH);

            loop {
                // break if read fails because it means broken pipe
                // we need to re-poll the pipe to get a new connection
                let event = ok_or_break!(gateway_pipe.read_string().await);
                //info!("got event: {}", event);

                let parsed_event = twilight_gateway::parse(
                    event,
                    EventTypeFlags::GUILD_CREATE
                        | EventTypeFlags::GUILD_DELETE
                        | EventTypeFlags::MESSAGE_CREATE
                        | EventTypeFlags::MESSAGE_DELETE
                        | EventTypeFlags::MESSAGE_UPDATE
                        | EventTypeFlags::READY
                        | EventTypeFlags::INTERACTION_CREATE
                        | EventTypeFlags::GUILD_UPDATE
                        | EventTypeFlags::CHANNEL_UPDATE,
                )
                .ok()
                .flatten();

                if let Some(parsed_event) = parsed_event {
                    let try_incoming_event: Result<IncomingEvent, _> = parsed_event.try_into();
                    if let Ok(incoming_event) = try_incoming_event {
                        assyst.metrics_handler.add_event();
                        let assyst_c = assyst.clone();
                        spawn(async move { handle_raw_event(assyst_c.clone(), incoming_event).await });
                    }
                }
            }

            err!("Connection to assyst-gateway lost, attempting reconnection");
        }
    });

    info!("Caching web download API URLs");
    let web_download_urls = get_web_download_api_urls(a.clone()).await.unwrap();
    info!("Got {} URLs to cache", web_download_urls.len());
    debug!(?web_download_urls);
    a.rest_cache_handler.set_web_download_urls(web_download_urls);

    loop {
        std::thread::sleep(Duration::from_secs(1));
    }
}
