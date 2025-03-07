use axum::{
    routing::{get, post},
    Router,
};

pub mod jamf;
pub mod routes;
use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tracing::{info, Level};
use tracing_subscriber::fmt;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() {
    // Set up tracing subscriber so we can see logs in stdout
    tracing_subscriber::registry()
        .with(EnvFilter::new(
            "info,tower_http=debug,axum=debug".to_string(),
        ))
        .with(
            fmt::layer()
                .with_target(true)
                .with_timer(fmt::time::uptime()) // Show uptime since application start
                .with_ansi(true) // Enable colors
                .pretty(), // Use a more readable format
        )
        .init();

    // Create routes
    let app = Router::new()
        .route("/", get(routes::hello::hello_world))
        .route(
            "/api/jamf/credentials",
            post(routes::credentials::credentials),
        )
        // NOTE: Because there are only computers in the Jamf instance, here I only get computers and not mobile devices
        .route("/api/jamf/devices", get(routes::devices::devices))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_request(DefaultOnRequest::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO)),
        );

    info!("Listening on 0.0.0.0:3000");
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
