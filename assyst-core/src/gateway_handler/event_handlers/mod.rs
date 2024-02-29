use crate::command::errors::{ExecutionError, TagParseError};
use crate::command::{Command, CommandCtxt};

pub mod guild_create;
pub mod guild_delete;
pub mod message_create;
pub mod message_delete;
pub mod message_update;
pub mod ready;

pub(super) async fn ctxt_exec(ctxt: &CommandCtxt<'_>, cmd: &(dyn Command + Send + Sync)) -> Result<(), ExecutionError> {
    if cmd.metadata().age_restricted {
        let channel_age_restricted = ctxt
            .assyst()
            .rest_cache_handler
            .channel_is_age_restricted(ctxt.data.channel_id)
            .await
            .unwrap_or(false);

        if !channel_age_restricted {
            return Err(ExecutionError::Parse(TagParseError::IllegalAgeRestrictedCommand));
        };
    };

    if cmd.metadata().send_processing {
        if let Err(e) = ctxt.reply("Processing...").await {
            Err(ExecutionError::Command(e))
        } else {
            cmd.execute(ctxt.clone()).await
        }
    } else {
        cmd.execute(ctxt.clone()).await
    }
}
