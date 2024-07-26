use twilight_http::Client as HttpClient;
use twilight_model::id::marker::WebhookMarker;
use twilight_model::id::Id;

use crate::config::config::LoggingWebhook;
use crate::config::CONFIG;

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
    if CONFIG.logging_webhooks.enable_webhooks {
        tokio::spawn(async move {
            let LoggingWebhook { id, token } = CONFIG.logging_webhooks.error.clone();

            let client = HttpClient::new(CONFIG.authentication.discord_token.clone());

            if id == 0 {
                tracing::error!("Failed to trigger error webhook: Error webhook ID is 0");
            } else {
                let webhook = client
                    .execute_webhook(Id::<WebhookMarker>::new(id), &token)
                    .content(&message);

                let _ = webhook
                    .await
                    .inspect_err(|e| tracing::error!("Failed to trigger error webhook: {}", e.to_string()));
            }
        });
    }
}
