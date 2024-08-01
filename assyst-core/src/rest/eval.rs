use anyhow::Context;
use assyst_common::config::CONFIG;
use assyst_common::eval::{FakeEvalBody, FakeEvalImageResponse, FakeEvalMessageData};
use assyst_common::util::filetype::get_sig;
use reqwest::Client;
use twilight_model::channel::Message;

pub async fn fake_eval(
    client: &Client,
    code: String,
    accept_image: bool,
    message: Option<&Message>,
    args: Vec<String>,
) -> anyhow::Result<FakeEvalImageResponse> {
    let response = client
        .post(format!("{}/eval", CONFIG.urls.eval))
        .query(&[("returnBuffer", accept_image)])
        .json(&FakeEvalBody {
            code,
            data: Some(FakeEvalMessageData { args, message }),
        })
        .send()
        .await?
        .bytes()
        .await?;

    if let Some(sig) = get_sig(&response) {
        Ok(FakeEvalImageResponse::Image(response.to_vec(), sig))
    } else {
        let text = std::str::from_utf8(&response).context("eval returned non-utf8 text response")?;
        Ok(FakeEvalImageResponse::Text(serde_json::from_str(text)?))
    }
}
