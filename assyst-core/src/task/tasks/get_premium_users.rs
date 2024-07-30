use assyst_common::config::CONFIG;
use assyst_common::err;
use tracing::info;

use crate::assyst::ThreadSafeAssyst;
use crate::rest::patreon::{Patron, PatronTier};

/// Synchronises Assyst with an updated list of patrons.
pub async fn get_premium_users(assyst: ThreadSafeAssyst) {
    let mut premium_users: Vec<Patron> = vec![];

    if !CONFIG.dev.disable_patreon_synchronisation {
        info!("Synchronising patron list");

        // get patron list and update in assyst
        let patrons = match crate::rest::patreon::get_patrons(assyst.clone()).await {
            Ok(p) => p,
            Err(e) => {
                err!("Failed to get patron list for synchronisation: {}", e.to_string());
                return;
            },
        };

        premium_users.extend(patrons.into_iter());

        info!("Synchronised patrons from Patreon: total {}", premium_users.len());
    }

    // todo: load premium users via entitlements once twilight supports this

    for i in CONFIG.dev.admin_users.iter() {
        premium_users.push(Patron {
            user_id: *i,
            tier: PatronTier::Tier4,
            _admin: true,
        })
    }

    assyst.update_premium_user_list(premium_users.clone());
}
