use anyhow::bail;
use assyst_common::config::config::LoggingWebhook;
use assyst_common::config::CONFIG;
use assyst_common::prometheus::Prometheus;
use assyst_common::{err, BOT_ID};
use assyst_database::model::free_tier_1_requests::FreeTier1Requests;
use assyst_database::model::user_votes::UserVotes;
use assyst_database::DatabaseHandler;
use axum::extract::State;
use axum::response::Redirect;
use axum::routing::{get, post};
use axum::{Json, Router};
use lazy_static::lazy_static;
use prometheus::TextEncoder;
use serde::Deserialize;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::spawn;
use tokio::sync::{Mutex, RwLock};
use twilight_http::Client as HttpClient;
use twilight_model::id::marker::{UserMarker, WebhookMarker};
use twilight_model::id::Id;

const FREE_TIER_1_REQUESTS_ON_VOTE: u64 = 15;
lazy_static! {
    static ref TOP_GG_VOTE_URL: String = format!("https://top.gg/bot/{}/vote", BOT_ID);
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TopGgWebhookBody {
    bot: String,
    user: String,
    r#type: String,
    is_weekend: bool,
    query: Option<String>,
}

#[derive(Clone)]
struct RouteState {
    pub database: Arc<RwLock<DatabaseHandler>>,
    pub http_client: Arc<HttpClient>,
    pub prometheus: Arc<Mutex<Prometheus>>,
}

async fn prometheus_metrics(State(route_state): State<RouteState>) -> String {
    route_state.prometheus.lock().await.update().await;

    let encoder = TextEncoder::new();
    let family = prometheus::gather();
    let response = encoder.encode_to_string(&family).expect("Encoding failed");
    response
}

async fn top_gg_webhook(State(route_state): State<RouteState>, Json(body): Json<TopGgWebhookBody>) {
    let user_id = match body.user.clone().parse::<u64>() {
        Ok(i) => i,
        Err(e) => {
            err!("Failed to parse user ID {} on vote: {}", body.user, e.to_string());
            return;
        },
    };

    if let Err(e) = FreeTier1Requests::change_free_tier_1_requests(
        &*route_state.database.read().await,
        user_id,
        FREE_TIER_1_REQUESTS_ON_VOTE,
    )
    .await
    {
        err!("Failed to add free tier 1 requests on vote: {}", e.to_string());
    } else {
        let user = match route_state
            .http_client
            .user(Id::<UserMarker>::new(user_id))
            .await
            .map(|u| u.model())
        {
            // better hope it can deserialize
            Ok(u) => u.await.unwrap(),
            Err(e) => {
                err!("Failed to get user object from user ID {}: {}", user_id, e.to_string());
                return;
            },
        };

        if let Err(_) = UserVotes::increment_user_votes(
            &*route_state.database.read().await,
            user_id,
            &user.name,
            &user.discriminator.to_string(),
        )
        .await
        .map_err(|e| err!("Failed to increment user {} votes on vote: {}", user_id, e.to_string()))
        {
            return;
        };

        let voter = match UserVotes::get_user_votes(&*route_state.database.read().await, user_id).await {
            Ok(v) => v,
            Err(e) => {
                err!(
                    "Failed to get voter after incrementing user votes for ID {}: {}",
                    user_id,
                    e.to_string()
                );
                return;
            },
        };

        if let Some(v) = voter {
            let message = format!(
                "{0}#{1} voted for Assyst on top.gg and got {2} free tier 1 requests!\n{0}#{1} has voted {3} total times.",
                user.name, user.discriminator, FREE_TIER_1_REQUESTS_ON_VOTE, v.count
            );

            let LoggingWebhook { id, token } = CONFIG.logging_webhooks.vote.clone();

            let _ = route_state
                .http_client
                .execute_webhook(Id::<WebhookMarker>::new(id), &token)
                .content(&message)
                .unwrap()
                .await;
        }
    }
}

/// Starts the webserver, providing bot list webhooking and prometheus services.
pub async fn run(
    database: Arc<RwLock<DatabaseHandler>>,
    http_client: Arc<HttpClient>,
    prometheus: Arc<Mutex<Prometheus>>,
) {
    let router = Router::new()
        .route("/", get(|| async { Redirect::permanent("https://jacher.io/assyst") }))
        .route("/topgg", get(|| async { Redirect::permanent(&TOP_GG_VOTE_URL) }))
        .route("/topgg", post(top_gg_webhook))
        .route("/metrics", get(prometheus_metrics))
        .with_state(RouteState {
            database,
            http_client,
            prometheus,
        });

    spawn(async move {
        let listener = TcpListener::bind(&format!("0.0.0.0:{}", CONFIG.authentication.top_gg_webhook_port))
            .await
            .unwrap();
        axum::serve(listener, router).await.unwrap();
    });
}
