use std::collections::HashMap;
use std::sync::OnceLock;

use tracing::info;
use twilight_model::application::command::Command as InteractionCommand;

use crate::assyst::ThreadSafeAssyst;
use crate::command::CommandMetadata;

use super::{fun, misc, services, wsi, TCommand};

macro_rules! declare_commands {
    ($($name:path),*) => {
        const RAW_COMMANDS: &[TCommand] = &[
            $(&$name as TCommand),*
        ];
    }
}

declare_commands!(
    fun::colour::colour_command,
    fun::findsong_command,
    fun::translation::bad_translate_command,
    fun::translation::translate_command,
    misc::enlarge_command,
    misc::eval_command,
    misc::exec_command,
    misc::help::help_command,
    misc::ping_command,
    misc::remind::remind_command,
    misc::run::run_command,
    misc::stats::stats_command,
    misc::tag::tag_command,
    misc::url_command,
    services::burntext_command,
    services::cooltext_command,
    services::download_command,
    services::r34_command,
    wsi::ahshit_command,
    wsi::aprilfools_command,
    wsi::bloom_command,
    wsi::blur_command,
    wsi::caption_command,
    wsi::resize_command
);

static COMMANDS: OnceLock<HashMap<&'static str, TCommand>> = OnceLock::new();

/// Prefer [`find_command_by_name`] where possible.
pub fn get_or_init_commands() -> &'static HashMap<&'static str, TCommand> {
    COMMANDS.get_or_init(|| {
        let mut map = HashMap::new();

        for &command in RAW_COMMANDS {
            let &CommandMetadata { name, aliases, .. } = command.metadata();
            map.insert(name, command);
            info!("Registering command {} (aliases={:?})", name, aliases);
            for alias in aliases {
                map.insert(alias, command);
            }
        }

        map
    })
}

/// Finds a command by its name.
pub fn find_command_by_name(name: &str) -> Option<TCommand> {
    get_or_init_commands().get(name).copied()
}

pub async fn register_interaction_commands(assyst: ThreadSafeAssyst) -> anyhow::Result<Vec<InteractionCommand>> {
    let commands = get_or_init_commands()
        .iter()
        .map(|x| x.1.as_interaction_command())
        .filter(|x| !x.name.is_empty())
        .collect::<Vec<_>>();

    // deduplicate out aliases
    let mut deduplicated_commands: Vec<InteractionCommand> = vec![];
    for command in commands {
        if !deduplicated_commands.iter().any(|x| x.name == command.name) {
            deduplicated_commands.push(command);
        }
    }

    let response = assyst
        .interaction_client()
        .set_global_commands(&deduplicated_commands)
        .await?
        .model()
        .await?;

    Ok(response)
}
