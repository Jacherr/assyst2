use assyst_common::err;
use assyst_database::model::active_guild_premium_entitlement::ActiveGuildPremiumEntitlement;
use tracing::info;

use crate::assyst::ThreadSafeAssyst;

pub async fn refresh_entitlements(assyst: ThreadSafeAssyst) {
    let clone = assyst.entitlements.lock().unwrap().clone();
    let entitlements = clone.iter().collect::<Vec<_>>();

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
