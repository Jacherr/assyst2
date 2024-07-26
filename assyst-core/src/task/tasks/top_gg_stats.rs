use assyst_common::err;
use tracing::debug;

use crate::assyst::ThreadSafeAssyst;
use crate::rest::top_gg::post_top_gg_stats as post_stats;

pub async fn post_top_gg_stats(assyst: ThreadSafeAssyst) {
    debug!("Updating stats on top.gg");

    if let Err(e) = post_stats(assyst).await {
        err!("Failed to post top.gg stats: {}", e.to_string());
    }

    debug!("Updated stats on top.gg");
}
