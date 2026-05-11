use anyhow::Result;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod submitter;
mod confirmer;
mod retry;
mod error;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "sol_relayer=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = sol_common::AppConfig::from_env()?;
    let pool = sqlx::PgPool::connect(&config.database_url).await?;
    let redis_client = redis::Client::open(config.redis_url.clone())?;

    tracing::info!("⚡ SolGrid TX Relayer started");
    tracing::info!("  RPC: {}", config.solana_rpc_url);

    // Main loop: consume TX requests from Redis queue
    loop {
        match process_next_tx(&pool, &redis_client, &config).await {
            Ok(true) => {} // Processed a TX
            Ok(false) => {
                // No TX in queue, sleep briefly
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            }
            Err(e) => {
                tracing::error!("Relayer error: {}", e);
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            }
        }
    }
}

async fn process_next_tx(
    pool: &sqlx::PgPool,
    redis_client: &redis::Client,
    config: &sol_common::AppConfig,
) -> Result<bool> {
    let mut conn = redis_client.get_multiplexed_async_connection().await?;

    // Blocking pop from TX queue (with 1s timeout)
    let result: Option<(String, String)> = redis::AsyncCommands::brpop(&mut conn, "solgrid:tx_queue", 1.0).await?;

    let payload = match result {
        Some((_, payload)) => payload,
        None => return Ok(false),
    };

    let tx_request: sol_common::TxRequest = serde_json::from_str(&payload)?;
    tracing::info!("Processing TX: {:?} for {}", tx_request.tx_type, tx_request.reference_id);

    // Submit transaction with retry
    let tx_result = retry::with_retry(5, || {
        submitter::submit_transaction(&tx_request, config)
    })
    .await;

    match tx_result {
        Ok(tx_hash) => {
            tracing::info!("✅ TX confirmed: {} ({})", tx_hash, tx_request.tx_type);

            // Record in DB
            sol_db::insert_transaction(
                pool,
                &tx_hash,
                &tx_request.tx_type.to_string(),
                tx_request.reference_id,
            )
            .await?;

            // Update the relevant entity with the tx hash
            match tx_request.tx_type {
                sol_common::TxType::RegisterProvider => {
                    sol_db::update_provider_onchain_tx(pool, tx_request.reference_id, &tx_hash)
                        .await?;
                }
                sol_common::TxType::CreateEscrow => {
                    sol_db::update_job_escrow_tx(pool, tx_request.reference_id, &tx_hash).await?;
                }
                sol_common::TxType::SubmitReceipt => {
                    sol_db::update_receipt_onchain_tx(pool, tx_request.reference_id, &tx_hash)
                        .await?;
                }
                sol_common::TxType::ReleaseEscrow => {
                    sol_db::update_job_settlement(pool, tx_request.reference_id, &tx_hash).await?;
                }
                sol_common::TxType::RefundEscrow => {
                    sol_db::update_job_status(pool, tx_request.reference_id, "failed").await?;
                }
            }

            // Publish result
            let result = sol_common::TxResult {
                tx_hash: tx_hash.clone(),
                tx_type: tx_request.tx_type,
                reference_id: tx_request.reference_id,
                success: true,
                error: None,
            };
            let _: Result<(), _> = redis::AsyncCommands::publish(
                &mut conn,
                "solgrid:tx_results",
                serde_json::to_string(&result).unwrap_or_default(),
            )
            .await;
        }
        Err(e) => {
            tracing::error!("❌ TX failed after retries: {} ({})", e, tx_request.tx_type);

            // Record failed transaction
            let tx_row = sol_db::insert_transaction(
                pool,
                "failed",
                &tx_request.tx_type.to_string(),
                tx_request.reference_id,
            )
            .await?;

            sol_db::update_transaction_status(pool, tx_row.id, "failed", Some(&e.to_string()))
                .await?;
        }
    }

    Ok(true)
}
