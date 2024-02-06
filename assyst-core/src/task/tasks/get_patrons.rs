use crate::assyst::ThreadSafeAssyst;
use tracing::{error, info};

/// Synchronises Assyst with an updated list of patrons.
pub async fn get_patrons(assyst: ThreadSafeAssyst) {
    info!("Synchronising patron list");

    // get patron list and update in assyst
    let patrons = match crate::rest::patreon::get_patrons(assyst.clone()).await {
        Ok(p) => p,
        Err(e) => {
            error!("Failed to get patron list for synchronisation: {}", e.to_string());
            return;
        },
    };

    assyst.update_patron_list(patrons.clone()).await;

    info!(
        "Synchronised patrons: {} (total {})",
        patrons
            .iter()
            .map(|x| x.user_id.to_string())
            .collect::<Vec<_>>()
            .join(", "),
        patrons.len()
    );
}
