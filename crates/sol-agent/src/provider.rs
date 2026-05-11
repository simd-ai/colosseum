//! Provider keypair handling and on-chain registration.

use std::path::{Path, PathBuf};
use std::str::FromStr;

use anyhow::{anyhow, Context, Result};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    native_token::LAMPORTS_PER_SOL,
    pubkey::Pubkey,
    signature::{read_keypair_file, write_keypair_file, Keypair, Signer},
    transaction::Transaction,
};

use sol_common::{AppConfig, RegisterProviderRequest};
use sol_onchain::{ids::ProgramIds, ix};

const MIN_BALANCE_LAMPORTS: u64 = LAMPORTS_PER_SOL / 10; // 0.1 SOL
const AIRDROP_LAMPORTS: u64 = LAMPORTS_PER_SOL;

pub struct ProviderIdentity {
    pub keypair: Keypair,
    pub rpc: RpcClient,
    pub ids: ProgramIds,
}

impl ProviderIdentity {
    /// Load (or generate) the agent's persisted Solana keypair, build an RPC
    /// client, and resolve the program IDs from config.
    pub fn load(config: &AppConfig) -> Result<Self> {
        let path = expand_path(&config.agent_keypair_path);
        let keypair = if path.exists() {
            read_keypair_file(&path).map_err(|e| {
                anyhow!("failed to read agent keypair at {}: {}", path.display(), e)
            })?
        } else {
            tracing::info!("Generating fresh agent keypair at {}", path.display());
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let kp = Keypair::new();
            write_keypair_file(&kp, &path).map_err(|e| {
                anyhow!("failed to write agent keypair to {}: {}", path.display(), e)
            })?;
            kp
        };

        let rpc = RpcClient::new_with_commitment(
            config.solana_rpc_url.clone(),
            CommitmentConfig::confirmed(),
        );

        let ids = ProgramIds {
            provider_registry: Pubkey::from_str(&config.provider_registry_program_id)
                .context("invalid PROVIDER_REGISTRY_PROGRAM_ID")?,
            job_escrow: Pubkey::from_str(&config.job_escrow_program_id)
                .context("invalid JOB_ESCROW_PROGRAM_ID")?,
            compute_receipts: Pubkey::from_str(&config.compute_receipts_program_id)
                .context("invalid COMPUTE_RECEIPTS_PROGRAM_ID")?,
        };

        Ok(Self { keypair, rpc, ids })
    }

    pub fn pubkey(&self) -> Pubkey {
        self.keypair.pubkey()
    }

    /// Ensure the agent has enough SOL to pay rent + tx fees. Devnet airdrop
    /// only — silently no-ops on networks that don't support airdrop.
    pub async fn ensure_funded(&self) -> Result<()> {
        let balance = self.rpc.get_balance(&self.pubkey()).await?;
        if balance >= MIN_BALANCE_LAMPORTS {
            tracing::info!(
                "Agent balance OK: {} SOL",
                balance as f64 / LAMPORTS_PER_SOL as f64
            );
            return Ok(());
        }

        tracing::info!(
            "Agent balance low ({:.4} SOL), requesting airdrop",
            balance as f64 / LAMPORTS_PER_SOL as f64
        );
        match self
            .rpc
            .request_airdrop(&self.pubkey(), AIRDROP_LAMPORTS)
            .await
        {
            Ok(sig) => {
                // Wait for confirmation
                for _ in 0..30 {
                    if let Ok(confirmed) = self.rpc.confirm_transaction(&sig).await {
                        if confirmed {
                            tracing::info!("Airdrop confirmed: {}", sig);
                            return Ok(());
                        }
                    }
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                }
                tracing::warn!("Airdrop submitted but not confirmed in time: {}", sig);
                Ok(())
            }
            Err(e) => {
                tracing::warn!("Airdrop failed (not on devnet?): {}", e);
                Ok(())
            }
        }
    }

    /// Submit `register_provider` on-chain. Returns Some(signature) on
    /// success, or None if the provider account already exists (in which
    /// case the agent has already registered in a previous run).
    pub async fn register_onchain(
        &self,
        name: &str,
        gpu_class: &str,
        gpu_count: u8,
        max_scu_per_epoch: u64,
    ) -> Result<Option<String>> {
        let (provider_pda, _bump) =
            sol_onchain::pda::provider_pda(&self.ids.provider_registry, &self.pubkey());

        // Already on-chain? Skip.
        if let Ok(Some(_acct)) = self.rpc.get_account_with_commitment(
            &provider_pda,
            CommitmentConfig::confirmed(),
        ).await.map(|r| r.value) {
            tracing::info!("Provider PDA {} already exists on-chain", provider_pda);
            return Ok(None);
        }

        let instruction = ix::register_provider_ix(
            &self.ids,
            &self.pubkey(),
            name,
            gpu_class,
            gpu_count,
            max_scu_per_epoch,
        )?;

        let blockhash = self.rpc.get_latest_blockhash().await?;
        let tx = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&self.pubkey()),
            &[&self.keypair],
            blockhash,
        );

        let sig = self
            .rpc
            .send_and_confirm_transaction(&tx)
            .await
            .context("on-chain register_provider failed")?;

        tracing::info!("✅ register_provider on-chain: {}", sig);
        Ok(Some(sig.to_string()))
    }
}

fn expand_path(p: &str) -> PathBuf {
    PathBuf::from(shellexpand::tilde(p).into_owned())
}

/// Build the off-chain registration request the agent POSTs to the scheduler
/// after submitting the on-chain TX.
pub fn build_register_request(
    pubkey: &Pubkey,
    onchain_tx: Option<String>,
    gpu_class: &str,
    gpu_count: u8,
    max_scu_per_epoch: u64,
) -> RegisterProviderRequest {
    let pubkey_str = pubkey.to_string();
    RegisterProviderRequest {
        pubkey: pubkey_str.clone(),
        name: format!("MockProvider-{}", &pubkey_str[..8]),
        gpu_class: gpu_class.to_string(),
        gpu_count: gpu_count as i16,
        max_scu_per_epoch: max_scu_per_epoch as i64,
        onchain_tx,
    }
}

/// POST the registration to the scheduler API. 409 Conflict is treated as
/// success (re-registration after restart).
pub async fn post_to_scheduler(
    config: &AppConfig,
    req: &RegisterProviderRequest,
) -> Result<()> {
    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{}/api/v1/providers/register", config.scheduler_url))
        .json(req)
        .send()
        .await?;

    let status = resp.status();
    if status.is_success() {
        tracing::info!("✅ Provider registered with scheduler");
        Ok(())
    } else if status.as_u16() == 409 {
        tracing::info!("Provider already known to scheduler, continuing");
        Ok(())
    } else {
        let body = resp.text().await.unwrap_or_default();
        Err(anyhow!("scheduler registration failed ({}): {}", status, body))
    }
}

#[allow(dead_code)]
pub fn _force_path_use(_: &Path) {}
