use std::collections::HashMap;
use std::sync::OnceLock;

use super::{misc, TCommand};

macro_rules! declare_commands {
    ($($name:path),*) => {
        const RAW_COMMANDS: &[TCommand] = &[
            $(&$name as TCommand),*
        ];
    }
}

declare_commands!(misc::remind_command, misc::e_command);

static COMMANDS: OnceLock<HashMap<&'static str, TCommand>> = OnceLock::new();

fn get_or_init_commands() -> &'static HashMap<&'static str, TCommand> {
    COMMANDS.get_or_init(|| {
        let mut map = HashMap::new();

        for &command in RAW_COMMANDS {
            let meta = command.metadata();
            map.insert(meta.name, command);
            for alias in meta.aliases {
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
