use anyhow::Result;
use sol_common::RegisterProviderRequest;

/// Register this mock provider with the scheduler API.
pub async fn register_provider(
    config: &sol_common::AppConfig,
    provider_pubkey: &str,
) -> Result<()> {
    let client = reqwest::Client::new();

    let req = RegisterProviderRequest {
        pubkey: provider_pubkey.to_string(),
        name: format!("MockProvider-{}", &provider_pubkey[..8]),
        gpu_class: "A100".to_string(),
        gpu_count: 4,
        max_scu_per_epoch: 1_000_000,
    };

    tracing::info!("📋 Registering provider: {} ({})", req.name, provider_pubkey);

    let resp = client
        .post(format!("{}/api/v1/providers/register", config.scheduler_url))
        .json(&req)
        .send()
        .await?;

    if resp.status().is_success() {
        tracing::info!("✅ Provider registered successfully");
    } else {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        // 409 Conflict means already registered — that's fine
        if status.as_u16() == 409 {
            tracing::info!("Provider already registered, continuing...");
        } else {
            tracing::error!("Failed to register provider ({}): {}", status, body);
        }
    }

    Ok(())
}
