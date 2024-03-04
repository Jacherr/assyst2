use assyst_common::config::config::LoggingWebhook;
use assyst_common::config::CONFIG;
use assyst_common::metrics_handler::MetricsHandler;
use assyst_common::{err, BOT_ID};
use assyst_database::model::free_tier_2_requests::FreeTier2Requests;
use assyst_database::model::user_votes::UserVotes;
use assyst_database::DatabaseHandler;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use lazy_static::lazy_static;
use prometheus::TextEncoder;
use serde::Deserialize;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::spawn;
use tokio::sync::RwLock;
use twilight_http::Client as HttpClient;
use twilight_model::id::marker::{UserMarker, WebhookMarker};
use twilight_model::id::Id;

const FREE_TIER_2_REQUESTS_ON_VOTE: u64 = 15;
lazy_static! {
    static ref TOP_GG_VOTE_URL: String = format!("https://top.gg/bot/{}/vote", BOT_ID);
}

struct AppError(anyhow::Error);

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, AppError>`. That way you don't need to do that manually.
impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TopGgWebhookBody {
    _bot: String,
    user: String,
    _type: String,
    _is_weekend: bool,
    _query: Option<String>,
}

#[derive(Clone)]
struct RouteState {
    pub database: Arc<RwLock<DatabaseHandler>>,
    pub http_client: Arc<HttpClient>,
    pub metrics_handler: Arc<MetricsHandler>,
}

#[axum::debug_handler]
async fn prometheus_metrics(State(route_state): State<RouteState>) -> String {
    route_state.metrics_handler.update().await;

    let encoder = TextEncoder::new();
    let family = prometheus::gather();
    encoder.encode_to_string(&family).expect("Encoding failed")
}

async fn top_gg_webhook(
    State(route_state): State<RouteState>,
    Json(body): Json<TopGgWebhookBody>,
) -> Result<(), AppError> {
    let user_id = body
        .user
        .clone()
        .parse::<u64>()
        .inspect_err(|e| err!("Failed to parse user id {}: {}", body.user, e.to_string()))?;

    FreeTier2Requests::new(user_id)
        .change_free_tier_2_requests(&*route_state.database.read().await, FREE_TIER_2_REQUESTS_ON_VOTE as i64)
        .await
        .inspect_err(|e| {
            err!(
                "Failed to add free tier 1 requests for user {}: {}",
                user_id,
                e.to_string()
            )
        })?;

    let user = route_state
        .http_client
        .user(Id::<UserMarker>::new(user_id))
        .await
        .map(|u| u.model())
        .inspect_err(|e| err!("Failed to get user object from user ID {}: {}", user_id, e.to_string()))?
        .await
        .inspect_err(|e| {
            err!(
                "Failed to deserialize user object for user ID {}: {}",
                user_id,
                e.to_string()
            )
        })?;

    UserVotes::increment_user_votes(
        &*route_state.database.read().await,
        user_id,
        &user.name,
        &user.discriminator.to_string(),
    )
    .await
    .inspect_err(|e| {
        err!(
            "Failed to increment user ID {} votes on vote: {}",
            user_id,
            e.to_string()
        )
    })?;

    let voter = UserVotes::get_user_votes(&*route_state.database.read().await, user_id)
        .await
        .inspect_err(|e| {
            err!(
                "Failed to get voter after incrementing user votes for ID {}: {}",
                user_id,
                e.to_string()
            )
        })?;

    if let Some(v) = voter {
        let message = format!(
            "{0}#{1} voted for Assyst on top.gg and got {2} free tier 1 requests!\n{0}#{1} has voted {3} total times.",
            user.name, user.discriminator, FREE_TIER_2_REQUESTS_ON_VOTE, v.count
        );

        let LoggingWebhook { id, token } = CONFIG.logging_webhooks.vote.clone();

        let _ = route_state
            .http_client
            .execute_webhook(Id::<WebhookMarker>::new(id), &token)
            .content(&message)
            .unwrap()
            .await;
    };

    Ok(())
}

/// Starts the webserver, providing bot list webhooking and prometheus services.
pub async fn run(
    database: Arc<RwLock<DatabaseHandler>>,
    http_client: Arc<HttpClient>,
    metrics_handler: Arc<MetricsHandler>,
) {
    let router = Router::new()
        .route("/", get(|| async { Redirect::permanent("https://jacher.io/assyst") }))
        .route("/topgg", get(|| async { Redirect::permanent(&TOP_GG_VOTE_URL) }))
        .route("/topgg", post(top_gg_webhook))
        .route("/metrics", get(prometheus_metrics))
        .with_state(RouteState {
            database,
            http_client,
            metrics_handler,
        });

    spawn(async move {
        let listener = TcpListener::bind(&format!("0.0.0.0:{}", CONFIG.authentication.top_gg_webhook_port))
            .await
            .unwrap();
        axum::serve(listener, router).await.unwrap();
    });
}
