use std::fmt::Display;
use std::future::Future;
use std::pin::Pin;

use crate::context::{Context, InnerContext};

use twilight_model::application::command::{Command as TwilightCommand, CommandOption, CommandType};
use twilight_model::guild::Permissions;
use twilight_model::id::Id;

pub type Response<T> = Box<dyn Fn(InnerContext<T>) -> Pin<Box<dyn Future<Output = anyhow::Result<()>>>>>;

pub struct Cmd<T> {
    pub context: Box<Context<T>>,
    pub data: TwilightCommand,
    pub response: Response<T>,
}

impl<T> Cmd<T> {
    #[must_use]
    pub fn new(context: Box<Context<T>>) -> Self {
        Self {
            context,
            data: TwilightCommand {
                application_id: None,
                default_member_permissions: None,
                description: String::new(),
                description_localizations: None,
                dm_permission: Some(true),
                guild_id: None,
                id: None,
                kind: CommandType::Unknown(255),
                name: String::new(),
                name_localizations: None,
                nsfw: Some(false),
                options: vec![],
                version: Id::new(1),
            },
            response: Box::new(|_| Box::pin(async { Ok(()) })),
        }
    }

    #[must_use]
    pub fn command(&self) -> &TwilightCommand {
        &self.data
    }

    #[must_use]
    pub fn guild_id(mut self, guild_id: u64) -> Self {
        self.data.guild_id = Some(Id::new(guild_id));
        self
    }

    #[must_use]
    pub fn clear_guild_id(mut self) -> Self {
        self.data.guild_id = None;
        self
    }

    #[must_use]
    pub fn name(mut self, name: impl Display) -> Self {
        self.data.name = format!("{name}");
        self
    }

    #[must_use]
    pub fn description(mut self, description: impl Display) -> Self {
        self.data.description = format!("{description}");
        self
    }

    #[must_use]
    pub fn chat_input(mut self) -> Self {
        self.data.kind = CommandType::ChatInput;
        self
    }

    #[must_use]
    pub fn message(mut self) -> Self {
        self.data.kind = CommandType::Message;
        self
    }

    #[must_use]
    pub fn user(mut self) -> Self {
        self.data.kind = CommandType::User;
        self
    }

    #[must_use]
    pub fn default_permissions(mut self, default_member_permissions: Permissions) -> Self {
        self.data.default_member_permissions = Some(default_member_permissions);
        self
    }

    #[must_use]
    pub fn clear_default_permissions(mut self) -> Self {
        self.data.default_member_permissions = None;
        self
    }

    #[must_use]
    pub fn dm(mut self, dm: bool) -> Self {
        self.data.dm_permission = Some(dm);
        self
    }

    #[must_use]
    pub fn nsfw(mut self, nsfw: bool) -> Self {
        self.data.nsfw = Some(nsfw);
        self
    }

    #[must_use]
    pub fn options(mut self, options: Vec<CommandOption>) -> Self {
        self.data.options = options;
        self
    }

    #[must_use]
    pub fn option(mut self, option: CommandOption) -> Self {
        self.data.options.push(option);
        self
    }

    #[must_use]
    pub fn respond_with<F>(mut self, f: F) -> Self
    where
        F: Fn(InnerContext<T>) -> Pin<Box<dyn Future<Output = anyhow::Result<()>>>> + 'static,
    {
        self.response = Box::new(f);
        self
    }
}
