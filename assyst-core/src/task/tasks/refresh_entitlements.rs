use assyst_common::err;
use assyst_database::model::active_guild_premium_entitlement::ActiveGuildPremiumEntitlement;
use tracing::info;
use twilight_model::id::marker::EntitlementMarker;
use twilight_model::id::Id;

use crate::assyst::ThreadSafeAssyst;

pub async fn refresh_entitlements(assyst: ThreadSafeAssyst) {
    let clone = assyst.entitlements.lock().unwrap().clone();
    let mut entitlements = clone.iter().collect::<Vec<_>>();
    entitlements.sort_by(|x, y| y.1.entitlement_id.cmp(&x.1.entitlement_id));
    let latest = entitlements.first();
    let additional = match latest {
        Some(l) => {
            match assyst
                .http_client
                .entitlements(assyst.application_id)
                .after(Id::<EntitlementMarker>::new(l.1.entitlement_id as u64))
                .await
            {
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
            }
        },
        None => vec![],
    };

    for a in additional {
        if !assyst.entitlements.lock().unwrap().contains_key(&(a.id.get() as i64)) {
            let active = match ActiveGuildPremiumEntitlement::try_from(a) {
                Ok(a) => a,
                Err(e) => {
                    err!("Error processing new entitlement: {e:?}");
                    continue;
                },
            };

            if let Err(e) = active.set(&assyst.database_handler).await {
                err!(
                    "Error adding new entitlement from later fetch for ID {}: {e:?}",
                    active.entitlement_id
                );
            };
            assyst.entitlements.lock().unwrap().insert(active.guild_id, active);
        }
    }

    for entitlement in entitlements {
        if entitlement.1.expiry_unix_ms != 0 && entitlement.1.expired() {
            assyst.entitlements.lock().unwrap().remove(entitlement.0);
            info!(
                "Removed expired entitlement {} (guild {})",
                entitlement.1.entitlement_id, entitlement.1.guild_id
            );
            if let Err(e) =
                ActiveGuildPremiumEntitlement::delete(&assyst.database_handler, entitlement.1.entitlement_id).await
            {
                err!(
                    "Error deleting existing entitlement {}: {e:?}",
                    entitlement.1.entitlement_id
                );
            }
        }
    }
}
