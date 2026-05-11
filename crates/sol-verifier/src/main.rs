use anyhow::Result;
use axum::Router;
use axum::routing::post;
use std::sync::Arc;
use tower_http::cors::{CorsLayer, Any};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod validator;
mod routes;
mod error;

use routes::AppState;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "sol_verifier=debug,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = sol_common::AppConfig::from_env()?;
    let port = config.verifier_port;

    let pool = sqlx::PgPool::connect(&config.database_url).await?;
    let state = Arc::new(AppState { pool, config });

    let app = Router::new()
        .route("/api/v1/verify", post(routes::verify_receipt))
        .route("/health", axum::routing::get(|| async { "OK" }))
        .layer(CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = format!("0.0.0.0:{}", port);
    tracing::info!("🔍 SolGrid Verification Service listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
