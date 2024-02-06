use crate::assyst::ThreadSafeAssyst;
use crate::rest::top_gg::post_top_gg_stats as post_stats;
use assyst_common::err;
use tracing::info;

pub async fn post_top_gg_stats(assyst: ThreadSafeAssyst) {
    info!("Updating stats on top.gg");

    if let Err(e) = post_stats(assyst).await {
        err!("Failed to post top.gg stats: {}", e.to_string());
    }

    info!("Updated stats on top.gg");
}
