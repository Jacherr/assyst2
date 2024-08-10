use assyst_common::err;
use tracing::debug;

use crate::assyst::ThreadSafeAssyst;
use crate::rest::top_gg::post_top_gg_stats as post_stats;

pub async fn post_top_gg_stats(assyst: ThreadSafeAssyst) {
    debug!("Updating stats on top.gg");

    if let Err(e) = post_stats(
        &assyst.reqwest_client,
        assyst.metrics_handler.guilds.with_label_values(&["guilds"]).get() as u64,
    )
    .await
    {
        err!("Failed to post top.gg stats: {}", e.to_string());
    }

    debug!("Updated stats on top.gg");
}
