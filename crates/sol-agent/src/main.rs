use anyhow::Result;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod provider;
mod simulator;
mod telemetry;
mod receipt;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "sol_agent=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = sol_common::AppConfig::from_env()?;

    // Generate a provider keypair for this agent
    let (signing_key, verifying_key) = sol_common::generate_keypair();
    let provider_pubkey = bs58::encode(verifying_key.as_bytes()).into_string();

    tracing::info!("🤖 SolGrid Mock Provider Agent starting");
    tracing::info!("  Provider pubkey: {}", provider_pubkey);

    // Step 1: Register with scheduler
    provider::register_provider(&config, &provider_pubkey).await?;

    // Step 2: Subscribe to job assignments and process them
    let redis_client = redis::Client::open(config.redis_url.clone())?;
    let http_client = reqwest::Client::new();

    tracing::info!("📡 Listening for job assignments...");

    // Use Redis pub/sub to receive job assignments
    let mut pubsub = redis_client.get_async_pubsub().await?;
    pubsub.subscribe("solgrid:job_assignments").await?;

    use redis::AsyncCommands;
    let mut msg_stream = pubsub.into_on_message();

    loop {
        use tokio_stream::StreamExt;
        match msg_stream.next().await {
            Some(msg) => {
                let payload: String = msg.get_payload().unwrap_or_default();
                tracing::info!("📥 Received job assignment: {}", payload);

                let assignment: serde_json::Value = match serde_json::from_str(&payload) {
                    Ok(v) => v,
                    Err(e) => {
                        tracing::error!("Failed to parse assignment: {}", e);
                        continue;
                    }
                };

                let job_id_str = assignment["job_id"].as_str().unwrap_or_default();
                let gpu_class = assignment["gpu_class"].as_str().unwrap_or("A100");
                let gpu_count = assignment["gpu_count"].as_u64().unwrap_or(4) as u8;
                let max_duration = assignment["max_duration_sec"].as_u64().unwrap_or(120) as u32;

                // Process the job
                let signing_key_clone = signing_key.clone();
                let pubkey_clone = provider_pubkey.clone();
                let http = http_client.clone();
                let scheduler_url = config.scheduler_url.clone();

                tokio::spawn(async move {
                    if let Err(e) = process_job(
                        &http,
                        &scheduler_url,
                        job_id_str,
                        &pubkey_clone,
                        gpu_class,
                        gpu_count,
                        max_duration,
                        &signing_key_clone,
                    )
                    .await
                    {
                        tracing::error!("Job processing failed: {}", e);
                    }
                });
            }
            None => {
                tracing::warn!("Redis subscription ended, reconnecting...");
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                break;
            }
        }
    }

    Ok(())
}

async fn process_job(
    http: &reqwest::Client,
    scheduler_url: &str,
    job_id_str: &str,
    provider_pubkey: &str,
    gpu_class: &str,
    gpu_count: u8,
    max_duration: u32,
    signing_key: &ed25519_dalek::SigningKey,
) -> Result<()> {
    let job_id: uuid::Uuid = job_id_str.parse()?;

    // Step 1: Simulate compute
    let duration = simulator::simulate_compute(gpu_class, max_duration).await;

    // Step 2: Generate telemetry
    telemetry::emit_telemetry(provider_pubkey, job_id, gpu_count, duration).await;

    // Step 3: Build and sign receipt
    let receipt_data = receipt::build_receipt(
        job_id,
        provider_pubkey,
        gpu_class,
        gpu_count,
        duration,
        signing_key,
    );

    // Step 4: Submit receipt to scheduler
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
