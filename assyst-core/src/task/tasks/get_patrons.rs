use crate::assyst::ThreadSafeAssyst;
use assyst_common::config::CONFIG;
use tracing::{error, info};
use twilight_model::id::marker::{GuildMarker, RoleMarker};
use twilight_model::id::Id;

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

    // update roles in the patron guild to remove the role from anybody that should not have it
    // (unsubscribed)
    let patron_guild_members = match assyst
        .http_client
        .guild_members(Id::<GuildMarker>::new(CONFIG.patreon.patron_guild_id))
        .await
    {
        Ok(m) => m,
        Err(e) => {
            error!(
                "Failed to get member list in patron guild for role synchronisation: {}",
                e.to_string()
            );
            return;
        },
    };

    let patron_guild_members = match patron_guild_members.model().await {
        Ok(m) => m,
        Err(e) => {
            error!(
                "Failed to get member list in patron guild for role synchronisation: {}",
                e.to_string()
            );
            return;
        },
    };

    let patron_ids = patrons.iter().map(|x| x.user_id).collect::<Vec<_>>();
    for member in patron_guild_members {
        if patron_ids.contains(&member.user.id.get()) {
            // ignore err case because there is a low chance the user already has this role, in which case its
            // fine
            let _ = assyst
                .http_client
                .add_guild_member_role(
                    Id::<GuildMarker>::new(CONFIG.patreon.patron_guild_id),
                    member.user.id,
                    Id::<RoleMarker>::new(CONFIG.patreon.patron_role_id),
                )
                .await;
        } else if member
            .roles
            .iter()
            .map(|x| x.get())
            .collect::<Vec<_>>()
            .contains(&CONFIG.patreon.patron_role_id)
        {
            // ignore err case for same reason
            let _ = assyst
                .http_client
                .remove_guild_member_role(
                    Id::<GuildMarker>::new(CONFIG.patreon.patron_guild_id),
                    member.user.id,
                    Id::<RoleMarker>::new(CONFIG.patreon.patron_role_id),
                )
                .await;
        }
    }

    info!("Synchronised patrons");
}
