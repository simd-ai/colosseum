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
pub async fn register_provider(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RegisterProviderRequest>,
) -> Result<Json<sol_db::ProviderRow>, ApiError> {
    // Check if provider already exists
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

    // Enqueue on-chain registration via Redis
    let tx_request = sol_common::TxRequest {
        tx_type: sol_common::TxType::RegisterProvider,
        reference_id: provider.id,
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

    tracing::info!("Provider registered: {} ({})", provider.name, provider.pubkey);
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
