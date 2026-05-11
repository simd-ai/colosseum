use std::sync::Arc;
use crate::state::AppState;

/// Background scheduler loop that periodically checks for pending jobs
/// and attempts to auto-assign them to available providers.
pub async fn scheduler_loop(state: Arc<AppState>) {
    tracing::info!("🔄 Scheduler loop started (10s interval)");

    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;

        match run_scheduler_tick(&state).await {
            Ok(assigned) => {
                if assigned > 0 {
                    tracing::info!("Scheduler tick: assigned {} jobs", assigned);
                }
            }
            Err(e) => {
                tracing::error!("Scheduler tick error: {}", e);
            }
        }
    }
}

async fn run_scheduler_tick(state: &AppState) -> anyhow::Result<usize> {
    let pending_jobs = sol_db::get_pending_jobs(&state.pool).await?;
    let mut assigned_count = 0;

    for job in pending_jobs {
        let providers = sol_db::list_active_providers_by_gpu(
            &state.pool,
            &job.gpu_class,
            job.gpu_count,
        )
        .await?;

        if let Some(provider) = providers.first() {
            sol_db::assign_job(&state.pool, job.id, provider.id).await?;

            // Notify via Redis pub/sub
            if let Ok(mut conn) = state.redis.get_multiplexed_async_connection().await {
                let assignment = serde_json::json!({
                    "job_id": job.id,
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

            assigned_count += 1;
            tracing::debug!("Auto-assigned job {} to provider {}", job.id, provider.id);
        }
    }

    Ok(assigned_count)
}
