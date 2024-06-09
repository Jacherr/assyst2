use std::collections::HashMap;
use std::sync::OnceLock;

use tracing::info;
use twilight_model::application::command::Command as InteractionCommand;

use crate::assyst::ThreadSafeAssyst;
use crate::command::CommandMetadata;

use super::{misc, services, wsi, TCommand};

macro_rules! declare_commands {
    ($($name:path),*) => {
        const RAW_COMMANDS: &[TCommand] = &[
            $(&$name as TCommand),*
        ];
    }
}

declare_commands!(
    misc::enlarge_command,
    misc::help::help_command,
    misc::ping_command,
    misc::remind_command,
    misc::stats::stats_command,
    misc::tag::tag_command,
    misc::url_command,
    services::burntext_command,
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
    // todo: dont register aliases
    let commands = get_or_init_commands()
        .iter()
        .map(|x| {
            let mut interaction_command = x.1.as_interaction_command();
            interaction_command.name = if interaction_command.name.is_empty() {
                "".to_owned()
            } else {
                x.0.to_owned().to_owned()
            };
            interaction_command
        })
        .filter(|y| !y.name.is_empty())
        .collect::<Vec<_>>();

    let response = assyst
        .interaction_client()
        .set_global_commands(&commands)
        .await?
        .model()
        .await?;

    Ok(response)
}
