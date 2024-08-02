use assyst_common::err;
use assyst_database::model::active_guild_premium_entitlement::ActiveGuildPremiumEntitlement;
use twilight_model::gateway::payload::incoming::EntitlementDelete;

use crate::assyst::ThreadSafeAssyst;

pub async fn handle(assyst: ThreadSafeAssyst, event: EntitlementDelete) {
    let Some(guild_id) = event.guild_id else {
        err!(
            "Deleted entitlement ID {} (guild {:?} user {:?}) has no associated guild!",
            event.id,
            event.guild_id,
            event.user_id
        );

        return;
    };

    if event.user_id.is_none() {
        err!(
            "Deleted entitlement ID {} (guild {:?} user {:?}) has no associated user!",
            event.id,
            event.guild_id,
            event.user_id
        );

        return;
    };

    assyst.entitlements.lock().unwrap().remove(&(guild_id.get() as i64));
    match ActiveGuildPremiumEntitlement::delete(&assyst.database_handler, event.id.get() as i64).await {
        Err(e) => {
            err!("Error deleting existing entitlement {}: {e:?}", event.id);
        },
        _ => {},
    }
}
