use assyst_common::err;
use assyst_common::macros::handle_log;
use assyst_database::model::active_guild_premium_entitlement::ActiveGuildPremiumEntitlement;
use tracing::info;
use twilight_model::gateway::payload::incoming::EntitlementCreate;
use twilight_model::guild::Guild;
use twilight_model::id::marker::GuildMarker;
use twilight_model::id::Id;

use crate::assyst::ThreadSafeAssyst;

pub async fn handle(assyst: ThreadSafeAssyst, event: EntitlementCreate) {
    let existing = assyst
        .entitlements
        .lock()
        .unwrap()
        .contains_key(&(event.id.get() as i64));

    if existing {
        err!(
            "Entitlement ID {} (guild {:?} user {:?}) was created but already exists!",
            event.id,
            event.guild_id,
            event.user_id
        );

        return;
    }

    // no expiry/created = test entitlement, requires special handling
    let active = match ActiveGuildPremiumEntitlement::try_from(event.0) {
        Err(e) => {
            err!("Error handling new entitlement: {e:?}");
            return;
        },
        Ok(a) => a,
    };

    match active.set(&assyst.database_handler).await {
        Err(e) => {
            err!("Error registering new entitlement {}: {e:?}", active.entitlement_id);
        },
        _ => {},
    };

    let expiry = active.expiry_unix_ms;
    let guild_id = Id::<GuildMarker>::new(active.guild_id as u64);
    let entitlement_id = active.entitlement_id;

    assyst.entitlements.lock().unwrap().insert(active.guild_id, active);

    let g: anyhow::Result<Guild> = match assyst.http_client.guild(guild_id).await {
        Ok(g) => g.model().await.map_err(std::convert::Into::into),
        Err(e) => Err(e.into()),
    };

    match g {
        Ok(g) => {
            handle_log(format!("New entitlement! Guild: {guild_id} - {}", g.name));
        },
        Err(_) => {
            handle_log(format!("New entitlement! Guild: {guild_id}"));
        },
    }

    info!("Registered new entitlement: {entitlement_id} for guild {guild_id} (expiry unix {expiry})",);
}
