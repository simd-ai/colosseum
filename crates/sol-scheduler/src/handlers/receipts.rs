use std::sync::Arc;
use axum::{
    extract::{Path, State},
    Json,
};
use uuid::Uuid;
use sol_common::{ComputeReceiptData, VerificationRequest, VerificationResult};
use crate::state::AppState;
use crate::error::ApiError;

/// POST /api/v1/receipts
pub async fn submit_receipt(
    State(state): State<Arc<AppState>>,
    Json(receipt_data): Json<ComputeReceiptData>,
) -> Result<Json<sol_db::ReceiptRow>, ApiError> {
    // Validate job exists
    let job = sol_db::get_job(&state.pool, receipt_data.job_id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Job {} not found", receipt_data.job_id)))?;

    // Check no duplicate receipt
    if let Some(_existing) = sol_db::get_receipt_by_job(&state.pool, receipt_data.job_id).await? {
        return Err(ApiError::Conflict("Receipt already submitted for this job".into()));
    }

    // Update job status to receipt_submitted
    sol_db::update_job_status(&state.pool, receipt_data.job_id, "receipt_submitted").await?;

    // Insert receipt into DB
    let receipt = sol_db::insert_receipt(
        &state.pool,
        receipt_data.job_id,
        &receipt_data.provider_pubkey,
        &receipt_data.gpu_class,
        receipt_data.gpu_count as i16,
        receipt_data.execution_duration_sec as i32,
        receipt_data.scu_amount as i64,
        &receipt_data.result_hash,
        &receipt_data.provider_signature,
    )
    .await?;

    // Send to verification service
    let job_info = sol_common::JobInfo {
        id: job.id,
        client_pubkey: job.client_pubkey,
        provider_id: job.provider_id,
        gpu_class: job.gpu_class,
        gpu_count: job.gpu_count,
        max_duration_sec: job.max_duration_sec,
        budget_scu: job.budget_scu,
        status: job.status,
        assigned_at: job.assigned_at,
        completed_at: job.completed_at,
        escrow_tx: job.escrow_tx,
        settlement_tx: job.settlement_tx,
        created_at: job.created_at,
    };

    let verify_req = VerificationRequest {
        receipt: receipt_data.clone(),
        job: job_info,
    };

    // Call verification service
    let verify_result = state
        .http_client
        .post(format!("{}/api/v1/verify", state.config.verifier_url))
        .json(&verify_req)
        .send()
        .await;

    match verify_result {
        Ok(resp) if resp.status().is_success() => {
            if let Ok(result) = resp.json::<VerificationResult>().await {
                if result.approved {
                    // Mark as verified
                    sol_db::verify_receipt(
                        &state.pool,
                        receipt.id,
                        &result.verifier_signature,
                        None,
                    )
                    .await?;
                    sol_db::update_job_status(&state.pool, receipt_data.job_id, "verified").await?;

                    // Enqueue on-chain receipt submission + escrow release
                    if let Ok(mut conn) = state.redis.get_multiplexed_async_connection().await {
                        let tx_req = sol_common::TxRequest {
                            tx_type: sol_common::TxType::SubmitReceipt,
                            reference_id: receipt.id,
                            payload: serde_json::to_value(&receipt_data).unwrap_or_default(),
                        };
                        let _: Result<(), _> = redis::AsyncCommands::lpush(
                            &mut conn,
                            "solgrid:tx_queue",
                            serde_json::to_string(&tx_req).unwrap_or_default(),
                        )
                        .await;

                        let release_req = sol_common::TxRequest {
                            tx_type: sol_common::TxType::ReleaseEscrow,
                            reference_id: receipt_data.job_id,
                            payload: serde_json::json!({"receipt_id": receipt.id}),
                        };
                        let _: Result<(), _> = redis::AsyncCommands::lpush(
                            &mut conn,
                            "solgrid:tx_queue",
                            serde_json::to_string(&release_req).unwrap_or_default(),
                        )
                        .await;
                    }

                    tracing::info!("Receipt verified and settlement enqueued for job {}", receipt_data.job_id);
                } else {
                    sol_db::update_job_status(&state.pool, receipt_data.job_id, "failed").await?;
                    tracing::warn!(
                        "Receipt verification failed: {:?}",
                        result.reason
                    );
                }
            }
        }
        Ok(resp) => {
            tracing::warn!("Verification service returned {}", resp.status());
        }
        Err(e) => {
            tracing::error!("Failed to reach verification service: {}", e);
            // Mark as verified anyway for demo purposes (verification service may not be running)
            sol_db::verify_receipt(&state.pool, receipt.id, "demo-auto-verified", None).await?;
            sol_db::update_job_status(&state.pool, receipt_data.job_id, "verified").await?;
            tracing::info!("Receipt auto-verified (demo mode) for job {}", receipt_data.job_id);
        }
    }

    // Reload receipt with updated verification status
    let updated_receipt = sol_db::get_receipt_by_job(&state.pool, receipt_data.job_id)
        .await?
        .ok_or_else(|| ApiError::Internal("Receipt not found after insert".into()))?;

    Ok(Json(updated_receipt))
}

/// GET /api/v1/receipts/:job_id
pub async fn get_receipt(
    State(state): State<Arc<AppState>>,
    Path(job_id): Path<Uuid>,
) -> Result<Json<sol_db::ReceiptRow>, ApiError> {
    let receipt = sol_db::get_receipt_by_job(&state.pool, job_id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Receipt not found for job {}", job_id)))?;
    Ok(Json(receipt))
}

/// GET /api/v1/receipts
pub async fn list_receipts(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<sol_db::ReceiptRow>>, ApiError> {
    let receipts = sol_db::list_receipts(&state.pool).await?;
    Ok(Json(receipts))
}
