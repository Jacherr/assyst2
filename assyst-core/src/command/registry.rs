use std::collections::HashMap;
use std::sync::OnceLock;

use tracing::info;

use crate::command::CommandMetadata;

use super::{misc, wsi, TCommand};

macro_rules! declare_commands {
    ($($name:path),*) => {
        const RAW_COMMANDS: &[TCommand] = &[
            $(&$name as TCommand),*
        ];
    }
}

declare_commands!(
    misc::remind_command,
    misc::enlarge_command,
    misc::ping_command,
    misc::url_command,
    misc::help_command,
    misc::stats_command,
    wsi::caption_command,
    misc::tag::tag_command
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
