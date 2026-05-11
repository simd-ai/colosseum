//! SolGrid Mock GPU Provider Agent.
//!
//! Lifecycle:
//!   1. Load or generate a Solana keypair (persisted to disk)
//!   2. Airdrop SOL on devnet if balance is low
//!   3. Submit `register_provider` on-chain
//!   4. POST the registration (with on-chain tx hash) to the scheduler API
//!   5. Subscribe to Redis `solgrid:job_assignments`
//!   6. For each assigned job: simulate compute, build + sign receipt, POST it
//!
//! Steps 1–4 happen exactly once per restart; the on-chain step is idempotent
//! (skipped if the provider PDA already exists).

use anyhow::Result;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod provider;
mod receipt;
mod simulator;
mod telemetry;

const GPU_CLASS: &str = "A100";
const GPU_COUNT: u8 = 4;
const MAX_SCU_PER_EPOCH: u64 = 1_000_000;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "sol_agent=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = sol_common::AppConfig::from_env()?;

    let identity = Arc::new(provider::ProviderIdentity::load(&config)?);
    let provider_pubkey = identity.pubkey().to_string();

    tracing::info!("🤖 SolGrid Mock Provider Agent starting");
    tracing::info!("  Provider pubkey: {}", provider_pubkey);

    // Step 2 — make sure we have lamports to pay tx fees.
    identity.ensure_funded().await?;

    // Step 3 — register on-chain (idempotent).
    let onchain_tx = identity
        .register_onchain(
            &format!("MockProvider-{}", &provider_pubkey[..8]),
            GPU_CLASS,
            GPU_COUNT,
            MAX_SCU_PER_EPOCH,
        )
        .await?;

    // Step 4 — tell the scheduler we exist.
    let req = provider::build_register_request(
        &identity.pubkey(),
        onchain_tx,
        GPU_CLASS,
        GPU_COUNT,
        MAX_SCU_PER_EPOCH,
    );
    provider::post_to_scheduler(&config, &req).await?;

    // Step 5 — subscribe to assignments and run forever.
    let redis_client = redis::Client::open(config.redis_url.clone())?;
    let http_client = reqwest::Client::new();

    tracing::info!("📡 Listening for job assignments...");
    let mut pubsub = redis_client.get_async_pubsub().await?;
    pubsub.subscribe("solgrid:job_assignments").await?;
    let mut msg_stream = pubsub.into_on_message();

    use tokio_stream::StreamExt;
    while let Some(msg) = msg_stream.next().await {
        let payload: String = msg.get_payload().unwrap_or_default();
        tracing::info!("📥 Received job assignment: {}", payload);

        let assignment: serde_json::Value = match serde_json::from_str(&payload) {
            Ok(v) => v,
            Err(e) => {
                tracing::error!("Failed to parse assignment: {}", e);
                continue;
            }
        };

        // Only handle assignments addressed to *this* provider; others belong
        // to sibling agents in the same Redis topic.
        let assigned_pubkey = assignment["provider_pubkey"].as_str().unwrap_or_default();
        if assigned_pubkey != provider_pubkey {
            tracing::debug!("Skipping assignment for other provider {}", assigned_pubkey);
            continue;
        }

        let job_id_str = assignment["job_id"].as_str().unwrap_or_default().to_string();
        let gpu_class = assignment["gpu_class"].as_str().unwrap_or(GPU_CLASS).to_string();
        let gpu_count = assignment["gpu_count"].as_u64().unwrap_or(GPU_COUNT as u64) as u8;
        let max_duration = assignment["max_duration_sec"].as_u64().unwrap_or(120) as u32;

        let identity_clone = identity.clone();
        let http = http_client.clone();
        let scheduler_url = config.scheduler_url.clone();
        let pubkey_clone = provider_pubkey.clone();

        tokio::spawn(async move {
            if let Err(e) = process_job(
                &http,
                &scheduler_url,
                &job_id_str,
                &pubkey_clone,
                &gpu_class,
                gpu_count,
                max_duration,
                &identity_clone.keypair,
            )
            .await
            {
                tracing::error!("Job processing failed: {:#}", e);
            }
        });
    }

    tracing::warn!("Redis subscription ended");
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn process_job(
    http: &reqwest::Client,
    scheduler_url: &str,
    job_id_str: &str,
    provider_pubkey: &str,
    gpu_class: &str,
    gpu_count: u8,
    max_duration: u32,
    keypair: &solana_sdk::signature::Keypair,
) -> Result<()> {
    let job_id: uuid::Uuid = job_id_str.parse()?;

    let duration = simulator::simulate_compute(gpu_class, max_duration).await;
    telemetry::emit_telemetry(provider_pubkey, job_id, gpu_count, duration).await;

    let receipt_data = receipt::build_receipt(
        job_id,
        provider_pubkey,
        gpu_class,
        gpu_count,
        duration,
        keypair,
    );

    tracing::info!("📤 Submitting receipt for job {}", job_id);
    let resp = http
        .post(format!("{}/api/v1/receipts", scheduler_url))
        .json(&receipt_data)
        .send()
        .await?;

    if resp.status().is_success() {
        tracing::info!("✅ Receipt accepted for job {}", job_id);
    } else {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        tracing::error!("❌ Receipt rejected ({}): {}", status, body);
    }

    Ok(())
}
