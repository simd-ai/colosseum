use std::sync::Arc;
use axum::{
    extract::{Path, State},
    Json,
};
use uuid::Uuid;
use sol_common::CreateJobRequest;
use crate::state::AppState;
use crate::error::ApiError;

/// POST /api/v1/jobs
pub async fn create_job(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateJobRequest>,
) -> Result<Json<sol_db::JobRow>, ApiError> {
    let job = sol_db::insert_job(
        &state.pool,
        &req.client_pubkey,
        &req.gpu_class,
        req.gpu_count,
        req.max_duration_sec,
        req.budget_scu,
    )
    .await?;

    // Enqueue escrow creation
    let tx_request = sol_common::TxRequest {
        tx_type: sol_common::TxType::CreateEscrow,
        reference_id: job.id,
        payload: serde_json::to_value(&req).unwrap_or_default(),
    };
    if let Ok(mut conn) = state.redis.get_multiplexed_async_connection().await {
        let _: Result<(), _> = redis::AsyncCommands::lpush(
            &mut conn,
            "solgrid:tx_queue",
            serde_json::to_string(&tx_request).unwrap_or_default(),
        )
        .await;
    }

    tracing::info!("Job created: {}", job.id);
    Ok(Json(job))
}

/// GET /api/v1/jobs
pub async fn list_jobs(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<sol_db::JobRow>>, ApiError> {
    let jobs = sol_db::list_jobs(&state.pool).await?;
    Ok(Json(jobs))
}

/// GET /api/v1/jobs/:id
pub async fn get_job(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<sol_db::JobRow>, ApiError> {
    let job = sol_db::get_job(&state.pool, id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Job {} not found", id)))?;
    Ok(Json(job))
}

/// POST /api/v1/jobs/:id/assign
pub async fn assign_job(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<sol_db::JobRow>, ApiError> {
    let job = sol_db::get_job(&state.pool, id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Job {} not found", id)))?;

    if job.status != "pending" {
        return Err(ApiError::BadRequest(format!(
            "Job {} is not in pending status (current: {})",
            id, job.status
        )));
    }

    // Find best available provider
    let providers = sol_db::list_active_providers_by_gpu(
        &state.pool,
        &job.gpu_class,
        job.gpu_count,
    )
    .await?;

    let provider = providers
        .first()
        .ok_or_else(|| ApiError::NotFound("No available provider matching requirements".into()))?;

    let updated_job = sol_db::assign_job(&state.pool, id, provider.id).await?;

    // Notify provider via Redis pub/sub
    if let Ok(mut conn) = state.redis.get_multiplexed_async_connection().await {
        let assignment = serde_json::json!({
            "job_id": id,
            "provider_id": provider.id,
            "provider_pubkey": provider.pubkey,
            "gpu_class": job.gpu_class,
            "gpu_count": job.gpu_count,
            "max_duration_sec": job.max_duration_sec,
            "budget_scu": job.budget_scu,
        });
        let _: Result<(), _> = redis::AsyncCommands::publish(
            &mut conn,
            "solgrid:job_assignments",
            serde_json::to_string(&assignment).unwrap_or_default(),
        )
        .await;
    }

    tracing::info!("Job {} assigned to provider {}", id, provider.id);
    Ok(Json(updated_job))
}
