pub mod providers;
pub mod jobs;
pub mod receipts;

use std::sync::Arc;
use axum::{extract::State, Json};
use crate::state::AppState;
use crate::error::ApiError;

/// GET /api/v1/stats — Dashboard statistics.
pub async fn stats(
    State(state): State<Arc<AppState>>,
) -> Result<Json<sol_common::DashboardStats>, ApiError> {
    let stats = sol_db::get_dashboard_stats(&state.pool).await?;
    Ok(Json(stats))
}
