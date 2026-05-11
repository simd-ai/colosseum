pub mod providers;
pub mod jobs;
pub mod receipts;

use std::sync::Arc;
use axum::{Router, routing::{get, post}};
use crate::state::AppState;
use crate::handlers;

pub fn provider_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/v1/providers/register", post(handlers::providers::register_provider))
        .route("/api/v1/providers", get(handlers::providers::list_providers))
        .route("/api/v1/providers/{id}", get(handlers::providers::get_provider))
}

pub fn job_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/v1/jobs", post(handlers::jobs::create_job))
        .route("/api/v1/jobs", get(handlers::jobs::list_jobs))
        .route("/api/v1/jobs/{id}", get(handlers::jobs::get_job))
        .route("/api/v1/jobs/{id}/assign", post(handlers::jobs::assign_job))
}

pub fn receipt_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/v1/receipts", post(handlers::receipts::submit_receipt))
        .route("/api/v1/receipts/{job_id}", get(handlers::receipts::get_receipt))
        .route("/api/v1/receipts", get(handlers::receipts::list_receipts))
}

pub fn stats_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/v1/stats", get(handlers::stats))
}
