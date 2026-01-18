use std::collections::HashMap;
use std::time::SystemTime;

use assyst_common::err;
use assyst_common::macros::handle_log;
use assyst_database::model::active_guild_premium_entitlement::ActiveGuildPremiumEntitlement;
use tracing::info;

use crate::assyst::ThreadSafeAssyst;

pub async fn refresh_entitlements(assyst: ThreadSafeAssyst) {
    let additional = match assyst.http_client.entitlements(assyst.application_id).await {
        Ok(x) => match x.model().await {
            Ok(e) => e,
            Err(e) => {
                err!("Failed to get potential new entitlements: {e:?}");
                vec![]
            },
        },
        Err(e) => {
            err!("Failed to get potential new entitlements: {e:?}");
            vec![]
        },
    };

    info!(
        "Entitlement fetch yielded {} entitlements ({})",
        additional.len(),
        additional
            .clone()
            .iter()
            .map(|x| x.guild_id.map(|x| x.get()).unwrap_or(0).to_string())
            .collect::<Vec<_>>()
            .join(", ")
    );

    for a in additional.clone() {
        if let Some(g) = a.guild_id
            && !assyst.entitlements.lock().unwrap().contains_key(&(g.get() as i64))
        {
            let active = match ActiveGuildPremiumEntitlement::try_from(a) {
                Ok(a) => a,
                Err(e) => {
                    err!("Error processing new entitlement: {e:?}");
                    continue;
                },
            };

            if active.expired() {
                use std::time::UNIX_EPOCH;

                let current = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();

                info!(
                    "Entitlement for guild {} has expired! Current time: {}, expiry unix: {}",
                    active.guild_id, current, active.expiry_unix_ms
                );

                continue;
            }

            if let Err(e) = active.set(&assyst.database_handler).await {
                err!("Error adding new entitlement for ID {}: {e:?}", active.entitlement_id);
            };
            handle_log(format!("New entitlement! Guild: {}", active.guild_id));

            assyst.entitlements.lock().unwrap().insert(active.guild_id, active);
        }
    }

    {
        let lock = assyst.entitlements.lock().unwrap();

        info!(
            "Active entitlements: {} ({})",
            lock.len(),
            lock.iter()
                .map(|x| x.1.guild_id.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }

    let db_entitlements = ActiveGuildPremiumEntitlement::get_all(&assyst.database_handler)
        .await
        .ok()
        .unwrap_or(HashMap::new());

    // remove entitlements from the db that are not in the rest response
    for entitlement in db_entitlements.values() {
        if !additional
            .iter()
            .any(|x| x.id.get() as i64 == entitlement.entitlement_id)
            || entitlement.expired()
        {
            assyst.entitlements.lock().unwrap().remove(&entitlement.entitlement_id);
            info!(
                "Removed expired entitlement {} (guild {})",
                entitlement.entitlement_id, entitlement.guild_id
            );
            if let Err(e) =
                ActiveGuildPremiumEntitlement::delete(&assyst.database_handler, entitlement.entitlement_id).await
            {
                err!(
                    "Error deleting existing entitlement {}: {e:?}",
                    entitlement.entitlement_id
                );
            }
        }
    }
}
