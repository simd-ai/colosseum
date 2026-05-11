use std::sync::Arc;
use axum::{
    extract::{Path, State},
    Json,
};
use uuid::Uuid;
use sol_common::RegisterProviderRequest;
use crate::state::AppState;
use crate::error::ApiError;

/// POST /api/v1/providers/register
///
/// The agent submits `register_provider` to Solana itself (so it can pay rent
/// from its own keypair), then POSTs here with the resulting tx signature.
/// The scheduler stores both the off-chain row and the on-chain pointer.
pub async fn register_provider(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RegisterProviderRequest>,
) -> Result<Json<sol_db::ProviderRow>, ApiError> {
    if let Some(_existing) = sol_db::get_provider_by_pubkey(&state.pool, &req.pubkey).await? {
        return Err(ApiError::Conflict("Provider already registered".into()));
    }

    let provider = sol_db::insert_provider(
        &state.pool,
        &req.pubkey,
        &req.name,
        &req.gpu_class,
        req.gpu_count,
        req.max_scu_per_epoch,
    )
    .await?;

    if let Some(tx) = &req.onchain_tx {
        sol_db::update_provider_onchain_tx(&state.pool, provider.id, tx).await?;
        tracing::info!(
            "Provider registered: {} ({}) tx={}",
            provider.name,
            provider.pubkey,
            tx
        );
    } else {
        tracing::info!(
            "Provider registered without on-chain tx: {} ({}) — agent should submit it",
            provider.name,
            provider.pubkey
        );
    }

    let provider = sol_db::get_provider(&state.pool, provider.id)
        .await?
        .ok_or_else(|| ApiError::Internal("provider vanished after insert".into()))?;
    Ok(Json(provider))
}

/// GET /api/v1/providers
pub async fn list_providers(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<sol_db::ProviderRow>>, ApiError> {
    let providers = sol_db::list_providers(&state.pool).await?;
    Ok(Json(providers))
}

/// GET /api/v1/providers/:id
pub async fn get_provider(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<sol_db::ProviderRow>, ApiError> {
    let provider = sol_db::get_provider(&state.pool, id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Provider {} not found", id)))?;
    Ok(Json(provider))
}
