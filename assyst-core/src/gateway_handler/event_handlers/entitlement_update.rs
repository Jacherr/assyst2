use assyst_common::err;
use assyst_common::macros::handle_log;
use assyst_database::model::active_guild_premium_entitlement::ActiveGuildPremiumEntitlement;
use tracing::info;
use twilight_model::gateway::payload::incoming::EntitlementUpdate;
use twilight_model::guild::Guild;
use twilight_model::id::marker::GuildMarker;
use twilight_model::id::Id;

use crate::assyst::ThreadSafeAssyst;

pub async fn handle(assyst: ThreadSafeAssyst, event: EntitlementUpdate) {
    // no expiry/created = test entitlement, requires special handling
    let new = match ActiveGuildPremiumEntitlement::try_from(event.0) {
        Err(e) => {
            err!("Error handling new entitlement: {e:?}");
            return;
        },
        Ok(a) => a,
    };
    let guild_id = Id::<GuildMarker>::new(new.guild_id as u64);
    let entitlement_id = new.entitlement_id;

    match new.update(&assyst.database_handler).await {
        Err(e) => {
            err!("Error updating existing entitlement {entitlement_id}: {e:?}");
        },
        _ => {},
    };

    let g: anyhow::Result<Guild> = match assyst.http_client.guild(guild_id).await {
        Ok(g) => g.model().await.map_err(std::convert::Into::into),
        Err(e) => Err(e.into()),
    };

    match g {
        Ok(g) => {
            handle_log(format!("Updated entitlement! Guild: {guild_id} - {}", g.name));
        },
        Err(_) => {
            handle_log(format!("Updated entitlement! Guild: {guild_id}"));
        },
    }

    assyst.entitlements.lock().unwrap().insert(guild_id.get() as i64, new);

    info!("Updated entitlement: {entitlement_id} for guild {guild_id}");
}
