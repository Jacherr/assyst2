use crate::assyst::ThreadSafeAssyst;
use crate::rest::patreon::{Patron, PatronTier};
use assyst_common::config::CONFIG;
use assyst_common::err;
use tracing::info;

/// Synchronises Assyst with an updated list of patrons.
pub async fn get_patrons(assyst: ThreadSafeAssyst) {
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

        assyst.update_patron_list(patrons.clone()).await;

        info!("Synchronised patrons: total {}", patrons.len());
    } else {
        let mut patrons: Vec<Patron> = vec![];

        for i in CONFIG.dev.admin_users.iter() {
            patrons.push(Patron {
                user_id: *i,
                tier: PatronTier::Tier4,
                _admin: true,
            })
        }

        assyst.update_patron_list(patrons.clone()).await;
    }
}
