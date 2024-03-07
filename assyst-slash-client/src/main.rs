#![warn(clippy::pedantic)]
#![allow(
    clippy::module_name_repetitions,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::missing_safety_doc
)]
use std::sync::Arc;

use command::Cmd;
use context::{Context, InnerContext};
use response::ResponseBuilder;
use serde::Deserialize;
use tokio::spawn;
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};
use twilight_gateway::{Event, EventTypeFlags, Intents, Shard, StreamExt};
use twilight_http::Client;

use twilight_model::application::command::CommandType;
use twilight_model::application::interaction::InteractionData::ApplicationCommand;
use utils::to_multimap;

#[derive(Deserialize, Clone)]
pub struct Cfg {
    pub token: String,
    pub guild_id: u64,
}
pub mod command;
pub mod context;
pub mod response;
pub mod utils;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cfg_file = std::fs::read_to_string("Config.toml").expect("missing Config.toml");
    let cfg: Cfg = toml::from_str(&cfg_file).expect("error parsing TOML");

    let client = Arc::new(Client::new(cfg.token.clone()));
    let config = twilight_gateway::Config::new(cfg.token.clone(), Intents::empty());
    let application_id = client.current_user_application().await?.model().await?.id;
    let interactions = client.interaction(application_id);

    let ctx = Context::new(client.clone(), application_id, cfg);

    let cmds = vec![ping(&ctx)];

    for (k, v) in to_multimap(cmds.iter().map(|x| (x.command().guild_id, x))) {
        let c = &*v.iter().map(|x| x.command().clone()).collect::<Vec<_>>();
        let names = v
            .iter()
            .map(|x| {
                format!(
                    "\x1b[34m{}::{}\x1b[0m",
                    match x.data.kind {
                        CommandType::ChatInput => "chat_input",
                        CommandType::Message => "message",
                        CommandType::User => "user",
                        _ => "unknown", // unreachable? don't be stupid when making commands
                    },
                    x.data.name
                )
            })
            .collect::<Vec<_>>()
            .join(", ");
        if let Some(g) = k {
            interactions.set_guild_commands(g, c).await?;
            println!("\x1b[1;32mRegister\x1b[0m Guild [\x1b[33m{g}\x1b[0m] with [{names}]");
        } else {
            interactions.set_global_commands(c).await?;
            println!("\x1b[1;32mRegister\x1b[0m global commands with [{names}]");
        }
    }

    let shards = twilight_gateway::create_recommended(&client, config, |_, b| b.build())
        .await?
        .collect::<Vec<_>>();

    let (tx, mut rx) = unbounded_channel::<Event>();
    let mut tasks = vec![];

    for shard in shards {
        tasks.push(spawn(runner(shard, tx.clone())))
    }

    while let Some(e) = rx.recv().await {
        process(ctx.clone(), &cmds, e).await;
    }

    Ok(())
}

pub async fn runner(mut shard: Shard, tx: UnboundedSender<Event>) {
    while let Some(Ok(item)) = shard.next_event(EventTypeFlags::all()).await {
        tx.send(item).unwrap();
    }
}

#[must_use]
pub fn ping(ctx: &Context<Cfg>) -> Cmd<Cfg> {
    Cmd::new(Box::new(ctx.clone()))
        .name("test")
        .chat_input()
        .description("waow")
        .guild_id(ctx.data.guild_id)
        .respond_with(|ctx| {
            Box::pin(async move {
                ctx.respond(ResponseBuilder::channel_message_with_source().content("ok!"))
                    .await
                    .unwrap();
                Ok(())
            })
        })
}

pub async fn process<T>(client: Context<T>, cmds: &[Cmd<T>], event: Event) {
    let mut i = match event {
        Event::InteractionCreate(i) => i.0,
        _ => return,
    };

    let d = if let Some(ApplicationCommand(d)) = std::mem::take(&mut i.data) {
        *d
    } else {
        println!("\x1b[1;33mwarning\x1b[0m: ignored non-command interaction");
        return;
    };

    let Some(cmd) = cmds.iter().find(|x| x.data.name == d.name) else {
        println!(
            "\x1b[1;33mwarning\x1b[0m: ignored unlistened interaction \x1b[34m{}\x1b[0m (trigger a cleanup?)",
            d.name
        );
        return;
    };

    let ictx = InnerContext::new(Box::new(client), Box::new(i), Box::new(d));

    if let Err(e) = (cmd.response)(ictx).await {
        println!("\x1b[1;31merror\x1b[0m: command {} failed: {e}", cmd.data.name);
    };
}
