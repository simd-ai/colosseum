use std::sync::Arc;
use axum::{extract::State, Json};
use sol_common::{VerificationRequest, VerificationResult};

pub struct AppState {
    pub pool: sqlx::PgPool,
    pub config: sol_common::AppConfig,
}

/// POST /api/v1/verify — Verify a compute receipt.
pub async fn verify_receipt(
    State(state): State<Arc<AppState>>,
    Json(req): Json<VerificationRequest>,
) -> Json<VerificationResult> {
    // Fetch registered provider pubkeys for validation
    let providers = sol_db::list_providers(&state.pool)
        .await
        .unwrap_or_default();
    let provider_pubkeys: Vec<String> = providers.iter().map(|p| p.pubkey.clone()).collect();

    let result = crate::validator::validate_receipt(&req.receipt, &req.job, &provider_pubkeys);

    if result.approved {
        tracing::info!("✅ Receipt approved for job {}", req.receipt.job_id);
    } else {
        tracing::warn!("❌ Receipt rejected for job {}: {:?}", req.receipt.job_id, result.reason);
    }

    Json(result)
}
