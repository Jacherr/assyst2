use assyst_common::err;
use assyst_common::macros::handle_log;
use assyst_database::model::active_guild_premium_entitlement::ActiveGuildPremiumEntitlement;
use tracing::info;
use twilight_model::gateway::payload::incoming::EntitlementCreate;
use twilight_model::guild::Guild;
use twilight_model::util::Timestamp;

use crate::assyst::ThreadSafeAssyst;

pub async fn handle(assyst: ThreadSafeAssyst, event: EntitlementCreate) {
    let Some(guild_id) = event.guild_id else {
        err!(
            "Created entitlement ID {} (guild {:?} user {:?}) has no associated guild!",
            event.id,
            event.guild_id,
            event.user_id
        );

        return;
    };

    let Some(user_id) = event.user_id else {
        err!(
            "Created entitlement ID {} (guild {:?} user {:?}) has no associated user!",
            event.id,
            event.guild_id,
            event.user_id
        );

        return;
    };

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
    let active = ActiveGuildPremiumEntitlement {
        entitlement_id: event.id.get() as i64,
        guild_id: guild_id.get() as i64,
        user_id: user_id.get() as i64,
        started_unix_ms: event
            .starts_at
            .unwrap_or(Timestamp::from_micros(0).unwrap())
            .as_micros()
            / 1000,
        expiry_unix_ms: event.ends_at.unwrap_or(Timestamp::from_micros(0).unwrap()).as_micros() / 1000,
    };

    match active.set(&assyst.database_handler).await {
        Err(e) => {
            err!("Error registering new entitlement {}: {e:?}", event.id);
        },
        _ => {},
    };

    let expiry = active.expiry_unix_ms;

    assyst
        .entitlements
        .lock()
        .unwrap()
        .insert(guild_id.get() as i64, active);

    let g: anyhow::Result<Guild> = match assyst.http_client.guild(guild_id).await {
        Ok(g) => g.model().await.map_err(|e| e.into()),
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

    info!(
        "Registered new entitlement: {} for guild {guild_id} (expiry unix {expiry})",
        event.id
    );
}
