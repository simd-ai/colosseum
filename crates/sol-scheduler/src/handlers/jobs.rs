use std::sync::Arc;
use axum::{
    extract::{Path, State},
    Json,
};
use uuid::Uuid;
use sol_common::{AttachEscrowTxRequest, CreateJobRequest};
use crate::state::AppState;
use crate::error::ApiError;

/// POST /api/v1/jobs
///
/// Creates a job in DB and returns it. The caller (sol-client CLI) is then
/// responsible for submitting `create_escrow` on-chain with the returned
/// job_id, and PATCHing the resulting tx hash via `/jobs/:id/escrow`.
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

    tracing::info!("Job created: {} (awaiting escrow)", job.id);
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

/// POST /api/v1/jobs/:id/escrow
///
/// Called by sol-client after it has funded the on-chain escrow vault. Stores
/// the tx hash on the job row so the dashboard can link to it.
pub async fn attach_escrow_tx(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<AttachEscrowTxRequest>,
) -> Result<Json<sol_db::JobRow>, ApiError> {
    let _job = sol_db::get_job(&state.pool, id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Job {} not found", id)))?;

    sol_db::update_job_escrow_tx(&state.pool, id, &req.escrow_tx).await?;
    tracing::info!("Job {} escrow tx attached: {}", id, req.escrow_tx);

    let job = sol_db::get_job(&state.pool, id)
        .await?
        .ok_or_else(|| ApiError::Internal("job vanished after update".into()))?;
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
        let _: Result<(), _> = redis::AsyncCommands::publish::<_, _, ()>(
            &mut conn,
            "solgrid:job_assignments",
            serde_json::to_string(&assignment).unwrap_or_default(),
        )
        .await;
    }

    tracing::info!("Job {} assigned to provider {}", id, provider.id);
    Ok(Json(updated_job))
}
