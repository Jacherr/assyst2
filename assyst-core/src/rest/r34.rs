use crate::assyst::ThreadSafeAssyst;
use anyhow::{bail, Context};
use rand::prelude::SliceRandom;
use serde::Deserialize;

static R34_URL: &str = "https://api.rule34.xxx/index.php?tags=";

#[derive(Deserialize, Clone)]
pub struct R34Result {
    pub file_url: String,
    pub score: i32,
}

pub async fn get_random_r34(assyst: ThreadSafeAssyst, tags: &str) -> anyhow::Result<R34Result> {
    let all = assyst
        .reqwest_client
        .get(format!("{}{}", R34_URL, &tags.replace(' ', "+")[..]))
        .query(&[
            ("page", "dapi"),
            ("s", "post"),
            ("q", "index"),
            ("json", "1"),
            ("limit", "1000"),
        ])
        .send()
        .await?
        .error_for_status()?
        .json::<Vec<R34Result>>()
        .await;

    if let Err(e) = all {
        if e.to_string().contains("EOF") {
            bail!("No results found")
        }
        Err(e.into())
    } else {
        let all = all.unwrap();

        let mut rng = rand::thread_rng();

        all.choose(&mut rng).cloned().context("No results found")
    }
}
