//! SolGrid Solana TX Relayer.
//!
//! Consumes the `solgrid:tx_queue` Redis queue and submits the two
//! verifier-signed transactions to Solana:
//!
//!   * `SubmitReceipt`  — anchors a verified compute receipt on-chain
//!   * `ReleaseEscrow`  — moves the escrowed SIMD tokens to the provider
//!
//! `RegisterProvider` and `CreateEscrow` are submitted directly by the agent
//! and the sol-client CLI respectively (each from their own wallet), so the
//! relayer skips them if they appear in the queue.

use std::time::Duration;

use anyhow::Result;
use redis::AsyncCommands;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod submitter;

use submitter::Relayer;

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
    let relayer = Relayer::from_config(&config)?;

    tracing::info!("⚡ SolGrid TX Relayer started");
    tracing::info!("  RPC: {}", config.solana_rpc_url);

    loop {
        match process_next_tx(&pool, &redis_client, &relayer).await {
            Ok(true) => {}
            Ok(false) => tokio::time::sleep(Duration::from_millis(500)).await,
            Err(e) => {
                tracing::error!("Relayer error: {:#}", e);
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
        }
    }
}

async fn process_next_tx(
    pool: &sqlx::PgPool,
    redis_client: &redis::Client,
    relayer: &Relayer,
) -> Result<bool> {
    let mut conn = redis_client.get_multiplexed_async_connection().await?;

    let result: Option<(String, String)> =
        conn.brpop("solgrid:tx_queue", 1.0).await?;

    let payload = match result {
        Some((_, payload)) => payload,
        None => return Ok(false),
    };

    let tx_request: sol_common::TxRequest = serde_json::from_str(&payload)?;
    tracing::info!(
        "Processing TX: {:?} for {}",
        tx_request.tx_type,
        tx_request.reference_id
    );

    match tx_request.tx_type {
        sol_common::TxType::SubmitReceipt => {
            handle_submit_receipt(pool, &mut conn, relayer, &tx_request).await?;
        }
        sol_common::TxType::ReleaseEscrow => {
            handle_release_escrow(pool, &mut conn, relayer, &tx_request).await?;
        }
        sol_common::TxType::RegisterProvider | sol_common::TxType::CreateEscrow => {
            tracing::warn!(
                "Skipping {:?} — that TX type is submitted directly by its originator",
                tx_request.tx_type
            );
        }
        sol_common::TxType::RefundEscrow => {
            // Not implemented in the demo flow.
            tracing::warn!("RefundEscrow not yet implemented in the relayer");
        }
    }

    Ok(true)
}

async fn handle_submit_receipt(
    pool: &sqlx::PgPool,
    conn: &mut redis::aio::MultiplexedConnection,
    relayer: &Relayer,
    tx_request: &sol_common::TxRequest,
) -> Result<()> {
    let receipt: sol_common::ComputeReceiptData =
        serde_json::from_value(tx_request.payload.clone())?;

    let sig = relayer.submit_receipt(&receipt).await?;
    let sig_str = sig.to_string();
    tracing::info!("✅ SubmitReceipt confirmed: {}", sig_str);

    sol_db::insert_transaction(pool, &sig_str, "submit_receipt", tx_request.reference_id)
        .await?;
    sol_db::update_receipt_onchain_tx(pool, tx_request.reference_id, &sig_str).await?;

    publish_result(conn, tx_request, &sig_str, true, None).await;
    Ok(())
}

async fn handle_release_escrow(
    pool: &sqlx::PgPool,
    conn: &mut redis::aio::MultiplexedConnection,
    relayer: &Relayer,
    tx_request: &sol_common::TxRequest,
) -> Result<()> {
    // reference_id is the job_id (uuid). Look up provider pubkey.
    let job = sol_db::get_job(pool, tx_request.reference_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("job {} not found", tx_request.reference_id))?;

    let provider_id = job
        .provider_id
        .ok_or_else(|| anyhow::anyhow!("job {} has no assigned provider", job.id))?;
    let provider = sol_db::get_provider(pool, provider_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("provider {} not found", provider_id))?;

    let sig = relayer
        .release_escrow(tx_request.reference_id, &provider.pubkey)
        .await?;
    let sig_str = sig.to_string();
    tracing::info!("✅ ReleaseEscrow confirmed: {}", sig_str);

    sol_db::insert_transaction(pool, &sig_str, "release_escrow", tx_request.reference_id)
        .await?;
    sol_db::update_job_settlement(pool, tx_request.reference_id, &sig_str).await?;
    sol_db::update_job_status(pool, tx_request.reference_id, "settled").await?;

    publish_result(conn, tx_request, &sig_str, true, None).await;
    Ok(())
}

async fn publish_result(
    conn: &mut redis::aio::MultiplexedConnection,
    tx_request: &sol_common::TxRequest,
    tx_hash: &str,
    success: bool,
    error: Option<String>,
) {
    let result = sol_common::TxResult {
        tx_hash: tx_hash.to_string(),
        tx_type: tx_request.tx_type,
        reference_id: tx_request.reference_id,
        success,
        error,
    };
    let _: Result<(), _> = conn
        .publish::<_, _, ()>(
            "solgrid:tx_results",
            serde_json::to_string(&result).unwrap_or_default(),
        )
        .await;
}
