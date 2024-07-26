use assyst_common::config::CONFIG;
use reqwest::multipart::{Form, Part};

use crate::assyst::ThreadSafeAssyst;

const NOT_SO_IDENTIFY_URL: &str = "https://notsobot.com/api/media/av/tools/identify";

pub mod notsoidentify {
    use serde::{Deserialize, Serialize};
    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct Song {
        pub album: NamedField,
        pub artists: Vec<NamedField>,
        pub title: String,
        pub platforms: Platform,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct NamedField {
        pub name: String,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct Platform {
        pub youtube: Option<YouTube>,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct YouTube {
        pub url: String,
    }
}

pub async fn identify_song_notsoidentify(
    assyst: ThreadSafeAssyst,
    search: String,
) -> anyhow::Result<Vec<notsoidentify::Song>> {
    let client = &assyst.reqwest_client;
    let formdata = Form::new();
    let formdata = formdata.part("url", Part::text(search));
    Ok(client
        .post(NOT_SO_IDENTIFY_URL)
        .header("authorization", CONFIG.authentication.notsoapi.to_string())
        .multipart(formdata)
        .send()
        .await?
        .error_for_status()?
        .json::<Vec<notsoidentify::Song>>()
        .await?)
}
