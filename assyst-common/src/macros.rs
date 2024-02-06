use crate::config::CONFIG;
use twilight_http::Client as HttpClient;
use twilight_model::id::marker::WebhookMarker;
use twilight_model::id::Id;

#[macro_export]
macro_rules! ok_or_break {
    ($expression:expr) => {
        match $expression {
            Ok(v) => v,
            Err(_) => break,
        }
    };
}

#[macro_export]
macro_rules! ok_or_continue {
    ($expression:expr) => {
        match $expression {
            Ok(v) => v,
            Err(_) => continue,
        }
    };
}

#[macro_export]
macro_rules! unwrap_enum_variant {
    ($expression:expr, $variant:path) => {
        match $expression {
            $variant(v) => v,
            _ => unreachable!(),
        }
    };
}

#[macro_export]
macro_rules! err {
    ($($t:tt)*) => {{
        use $crate::macros::handle_log;
        let msg = format!($($t)*);
        tracing::error!("{}", &msg);

        handle_log(format!("Error: ```{}```", msg));
    }}
}

pub fn handle_log(message: String) {
    tokio::spawn(async move {
        let parts = CONFIG.logging_webhooks.error.split("/").collect::<Vec<_>>();
        let (token, id) = (
            *parts.iter().last().unwrap(),
            *parts.iter().nth(parts.len() - 2).unwrap(),
        );

        let client = HttpClient::new(CONFIG.authentication.discord_token.clone());
        let _ = client
            .execute_webhook(Id::<WebhookMarker>::new(id.parse::<u64>().unwrap()), token)
            .content(&message)
            .unwrap()
            .await;
    });
}
