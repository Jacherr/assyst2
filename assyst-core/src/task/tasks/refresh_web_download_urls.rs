use crate::assyst::ThreadSafeAssyst;
use crate::rest::web_media_download::get_web_download_api_urls;
use assyst_common::err;
use tracing::info;

pub async fn refresh_web_download_urls(assyst: ThreadSafeAssyst) {
    info!("Updating web download source URLs");

    let urls = get_web_download_api_urls(assyst.clone()).await;

    if let Ok(ref new) = urls {
        info!("Updated web download source URLs: got {} urls", new.len());
        assyst.rest_cache_handler.set_web_download_urls(new.clone());
    } else if let Err(e) = urls {
        err!("Error updating web download source URLs: {}", e.to_string())
    };
}
