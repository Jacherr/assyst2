use crate::assyst::ThreadSafeAssyst;
use crate::rest::web_media_download::get_web_download_api_urls;
use assyst_common::err;
use tracing::info;

pub async fn refresh_web_download_urls(assyst: ThreadSafeAssyst) {
    info!("Updating web download source URLs");

    let old = assyst
        .rest_cache_handler
        .get_web_download_urls()
        .iter()
        .map(|x| (*(x.clone())).clone())
        .collect::<Vec<_>>();

    let urls = get_web_download_api_urls(assyst.clone()).await;

    if let Ok(ref new) = urls {
        let mut removed = 0;
        let mut added = 0;

        println!("{:?}", old);
        println!("{:?}", new);

        for url in new {
            if !old.contains(url) {
                added += 1;
            }
        }

        for url in old {
            if !new.contains(&url) {
                removed += 1;
            }
        }

        info!(
            "Updated web download source URLs: got {} urls (removed={removed}, added={added})",
            new.len()
        );
        assyst.rest_cache_handler.set_web_download_urls(new.clone());
    } else if let Err(e) = urls {
        err!("Error updating web download source URLs: {}", e.to_string())
    };
}
