use assyst_common::err;
use assyst_common::util::discord::{dm_message_link, message_link};
use assyst_database::model::reminder::Reminder;
use twilight_model::channel::message::AllowedMentions;
use twilight_model::id::marker::{ChannelMarker, UserMarker};
use twilight_model::id::Id;

use crate::assyst::ThreadSafeAssyst;

// 30 seconds
pub static FETCH_INTERVAL: i64 = 30000;

async fn process_single_reminder(assyst: ThreadSafeAssyst, reminder: &Reminder) -> anyhow::Result<()> {
    assyst
        .http_client
        .create_message(Id::<ChannelMarker>::new(reminder.channel_id as u64))
        .allowed_mentions(Some(&AllowedMentions {
            parse: vec![],
            replied_user: false,
            roles: vec![],
            users: vec![Id::<UserMarker>::new(reminder.user_id as u64)],
        }))
        .content(&format!(
            "<@{}> Reminder: {}\n{}",
            reminder.user_id,
            reminder.message,
            // may not be set in a guild or have a message id
            if reminder.guild_id != 0 && reminder.message_id != 0 {
                message_link(
                    reminder.guild_id as u64,
                    reminder.channel_id as u64,
                    reminder.message_id as u64,
                )
            } else if reminder.guild_id == 0 && reminder.message_id != 0 {
                dm_message_link(reminder.channel_id as u64, reminder.message_id as u64)
            } else {
                "".to_owned()
            }
        ))
        .await?;

    Ok(())
}

async fn process_reminders(assyst: ThreadSafeAssyst, reminders: Vec<Reminder>) -> Result<(), anyhow::Error> {
    if reminders.len() < 1 {
        return Ok(());
    }

    for reminder in &reminders {
        if let Err(e) = process_single_reminder(assyst.clone(), &reminder).await {
            err!("Failed to process reminder: {:?}", e);
        }

        // Once we're done, delete them from database
        reminder.remove(&assyst.database_handler).await?;
    }

    Ok(())
}

pub async fn handle_reminders(assyst: ThreadSafeAssyst) {
    let reminders = Reminder::fetch_expiring_max(&assyst.database_handler, FETCH_INTERVAL).await;

    match reminders {
        Ok(reminders) => {
            if let Err(e) = process_reminders(assyst.clone(), reminders).await {
                err!("Processing reminder queue failed: {:?}", e);
            }
        },
        Err(e) => {
            err!("Fetching reminders failed: {:?}", e);
        },
    }
}
