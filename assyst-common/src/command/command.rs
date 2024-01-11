use std::time::Duration;

use super::argument::{Argument, ParsedArgument};
use super::category::Category;

/// Main command definition. Contains all key details about command definitions, for both gateway
/// and slash command use.
///
/// Declared commands are used by the gateway, slash command and core crate for processing.
pub struct Command {
    /// Main name for the command. This is the 'ID' for the command,
    /// and is used in the database etc. to reference the command.
    pub name: String,
    /// Aliases are alternative names for the command, and only used when invoking
    /// the command. Usually shorthand versions of the command name.
    pub aliases: Vec<String>,
    /// This is a hard-coded global flag to say if this command is disabled for non-developer users.
    /// May be used for in-development commands, or commands with known faults.
    pub disabled: bool,
    /// If the command can only be used in age-restricted channels.
    pub age_restricted: bool,
    /// How fast the command can be used in a single guild, independent of user.
    pub cooldown: Duration,
    /// Command category is used in the help command, and determines which file the command
    /// is defined in.
    pub category: Category,
    /// Command arguments, defined in the order in which they appear in the command.
    pub arguments: Vec<Argument>,
    /// Command description, used in the help command.
    pub descripton: String,
    /// Command usage examples, used in the help command.
    pub examples: Vec<String>,
    /// Command usage syntax, used in the help command.
    pub usage: String,
    /// If this command works in direct messages (DMs).
    pub supported_in_dms: bool,
}

/// A command after being parsed by the gateway or slash client.
pub struct ParsedCommand {
    /// The prefix used when calling this command. Can be the guild-specific prefix, any prefix
    /// override, or mentioning the bot.
    pub prefix: String,
    /// The invocation name used to call the command. This could be the command name, or any of its
    /// aliases.
    pub invoked_name: String,
    /// A Vec of arguments the command was called with.
    pub arguments: Vec<ParsedArgument>,
}

/// Builder utility structure to create a [Command] object.
pub struct CommandBuilder {
    name: Option<String>,
    aliases: Vec<String>,
    disabled: bool,
    age_restricted: bool,
    cooldown: Option<Duration>,
    category: Option<Category>,
    arguments: Vec<Argument>,
    description: Option<String>,
    examples: Vec<String>,
    usage: Option<String>,
    supported_in_dms: bool,
}
impl CommandBuilder {
    pub fn new() -> Self {
        Self {
            name: None,
            aliases: vec![],
            disabled: false,
            age_restricted: false,
            cooldown: None,
            category: None,
            arguments: vec![],
            description: None,
            examples: vec![],
            usage: None,
            supported_in_dms: false,
        }
    }

    pub fn build(&self) -> Command {
        Command {
            name: self.name.clone().expect("name is required in Command object"),
            aliases: self.aliases.clone(),
            disabled: self.disabled,
            age_restricted: self.age_restricted,
            cooldown: self.cooldown.expect("cooldown is required in Command object"),
            category: self.category.clone().expect("category is required in Command object"),
            arguments: self.arguments.clone(),
            descripton: self
                .description
                .clone()
                .expect("description is required in Command object"),
            examples: self.examples.clone(),
            usage: self.usage.clone().expect("usage is required in Command object"),
            supported_in_dms: self.supported_in_dms,
        }
    }

    pub fn name(&mut self, name: &str) -> &mut Self {
        self.name = Some(name.to_owned());
        self
    }

    pub fn alias(&mut self, alias: &str) -> &mut Self {
        self.aliases.push(alias.to_owned());
        self
    }

    pub fn disabled(&mut self, disabled: bool) -> &mut Self {
        self.disabled = disabled;
        self
    }

    pub fn age_restricted(&mut self, age_restricted: bool) -> &mut Self {
        self.age_restricted = age_restricted;
        self
    }

    pub fn cooldown(&mut self, cooldown: Duration) -> &mut Self {
        self.cooldown = Some(cooldown);
        self
    }

    pub fn category(&mut self, category: Category) -> &mut Self {
        self.category = Some(category);
        self
    }

    pub fn argument(&mut self, argument: Argument) -> &mut Self {
        self.arguments.push(argument);
        self
    }

    pub fn description(&mut self, description: &str) -> &mut Self {
        self.description = Some(description.to_owned());
        self
    }

    pub fn example(&mut self, example: &str) -> &mut Self {
        self.examples.push(example.to_owned());
        self
    }

    pub fn usage(&mut self, usage: &str) -> &mut Self {
        self.usage = Some(usage.to_owned());
        self
    }

    pub fn supported_in_dms(&mut self, supported_in_dms: bool) -> &mut Self {
        self.supported_in_dms = supported_in_dms;
        self
    }
}
