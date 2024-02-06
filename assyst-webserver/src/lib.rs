use assyst_common::config::CONFIG;
use assyst_database::DatabaseHandler;
use axum::routing::get;
use axum::Router;
use prometheus::TextEncoder;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::spawn;
use tokio::sync::RwLock;
use tracing::info;
use twilight_http::Client as HttpClient;

async fn root() -> &'static str {
    "Test"
}

async fn prometheus() -> String {
    let encoder = TextEncoder::new();
    let family = prometheus::gather();
    let response = encoder.encode_to_string(&family).expect("Encoding failed");
    response
}

/// Starts the webserver, providing bot list webhooking and prometheus services.
pub async fn run(database: Arc<RwLock<DatabaseHandler>>, http_client: Arc<HttpClient>) {
    let router = Router::new().route("/", get(root)).route("/metrics", get(prometheus));

    spawn(async move {
        info!("Starting assyst-webserver");

        let listener = TcpListener::bind(&format!("0.0.0.0:{}", CONFIG.authentication.top_gg_webhook_port))
            .await
            .unwrap();
        axum::serve(listener, router).await.unwrap();
    });
}
