use std::sync::Arc;
use anyhow::Result;
use axum::Router;
use tower_http::cors::{CorsLayer, Any};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod routes;
mod handlers;
mod services;
mod state;
mod error;

use state::AppState;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "sol_scheduler=debug,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load config
    let config = sol_common::AppConfig::from_env()?;
    let port = config.scheduler_port;

    // Create app state
    let state = AppState::new(config).await?;
    let shared_state = Arc::new(state);

    // Spawn background scheduler tick
    let scheduler_state = shared_state.clone();
    tokio::spawn(async move {
        services::scheduler::scheduler_loop(scheduler_state).await;
    });

    // Build router
    let app = Router::new()
        .merge(routes::provider_routes())
        .merge(routes::job_routes())
        .merge(routes::receipt_routes())
        .merge(routes::stats_routes())
        .layer(CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any))
        .layer(TraceLayer::new_for_http())
        .with_state(shared_state);

    // Start server
    let addr = format!("0.0.0.0:{}", port);
    tracing::info!("🚀 SolGrid Scheduler API listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
