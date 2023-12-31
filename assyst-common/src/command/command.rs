use std::time::Duration;

use super::argument::Argument;

/// Main command definition. Contains all key details about command definitions, for both gateway and slash command use.
/// 
/// Commands declared here are used by the gateway, slash command and core crate for processing.
pub struct Command {
    /// Main name for the command. This is the 'ID' for the command,
    /// and is used in the database etc. to reference the command.
    pub name: String,
    /// Aliases are alternative names for the command, and only used when invoking
    /// the command. Usually shorthand versions of the command name.
    pub aliases: Vec<String>,
    /// This is a hard-coded global flag to say if this command is disabled for non-developer users. May be used
    /// for in-development commands, or commands with known faults.
    pub disabled: bool,
    /// If the command can only be used in age-restricted channels.
    pub age_restricted: bool,
    /// How fast the command can be used in a single guild, independent of user.
    pub cooldown: Duration,
    /// Command category is used in the help command, and determines which file the command
    /// is defined in.
    pub category: String,
    /// Command arguments, defined in the order in which they appear in the command.
    pub arguments: Vec<Argument>
}