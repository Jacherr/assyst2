use crate::command::{CommandCtxt, TCommand};

pub mod channel_update;
pub mod guild_create;
pub mod guild_delete;
pub mod guild_update;
pub mod message_create;
pub mod message_delete;
pub mod message_update;
pub mod ready;

pub fn after_command_execution_success(ctxt: CommandCtxt<'_>, command: TCommand) {
    ctxt.assyst().metrics_handler.add_command();
    ctxt.assyst()
        .metrics_handler
        .add_individual_command_usage(command.metadata().name)
}
