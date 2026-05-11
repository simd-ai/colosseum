//! Real on-chain transaction submission for verifier-side operations.
//!
//! The relayer only signs and submits the two transactions that require the
//! verifier authority — `submit_receipt` and `release_escrow`. The agent and
//! sol-client CLI submit their own transactions for `register_provider` and
//! `create_escrow` directly.

use anyhow::{anyhow, Context, Result};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
    signature::{read_keypair_file, Keypair, Signature, Signer},
    transaction::Transaction,
};
use spl_associated_token_account::{
    get_associated_token_address,
    instruction::create_associated_token_account_idempotent,
};
use std::str::FromStr;

use sol_common::{AppConfig, ComputeReceiptData};
use sol_onchain::{ids::ProgramIds, ix};

pub struct Relayer {
    pub rpc: RpcClient,
    pub verifier: Keypair,
    pub ids: ProgramIds,
    pub mint: Pubkey,
}

impl Relayer {
    pub fn from_config(config: &AppConfig) -> Result<Self> {
        let rpc = RpcClient::new_with_commitment(
            config.solana_rpc_url.clone(),
            CommitmentConfig::confirmed(),
        );

        let verifier = load_keypair(&config.verifier_keypair_path)
            .with_context(|| {
                format!(
                    "loading verifier keypair from {}",
                    config.verifier_keypair_path
                )
            })?;

        let ids = ProgramIds {
            provider_registry: Pubkey::from_str(&config.provider_registry_program_id)
                .context("invalid PROVIDER_REGISTRY_PROGRAM_ID")?,
            job_escrow: Pubkey::from_str(&config.job_escrow_program_id)
                .context("invalid JOB_ESCROW_PROGRAM_ID")?,
            compute_receipts: Pubkey::from_str(&config.compute_receipts_program_id)
                .context("invalid COMPUTE_RECEIPTS_PROGRAM_ID")?,
        };

        let mint = Pubkey::from_str(&config.grid_token_mint)
            .context("invalid GRID_TOKEN_MINT")?;

        tracing::info!("Relayer verifier pubkey: {}", verifier.pubkey());
        tracing::info!("Relayer mint:            {}", mint);
        tracing::info!("provider_registry:       {}", ids.provider_registry);
        tracing::info!("job_escrow:              {}", ids.job_escrow);
        tracing::info!("compute_receipts:        {}", ids.compute_receipts);

        Ok(Self {
            rpc,
            verifier,
            ids,
            mint,
        })
    }

    /// Build, sign, send, and confirm a `submit_receipt` transaction.
    pub async fn submit_receipt(
        &self,
        receipt: &ComputeReceiptData,
    ) -> Result<Signature> {
        let provider = Pubkey::from_str(&receipt.provider_pubkey)
            .context("receipt.provider_pubkey is not a valid Solana pubkey")?;

        let job_id = job_id_from_uuid(&receipt.job_id);
        let result_hash: [u8; 32] = decode_fixed(&receipt.result_hash, "result_hash")?;
        let provider_signature: [u8; 64] =
            decode_fixed(&receipt.provider_signature, "provider_signature")?;

        let ix = ix::submit_receipt_ix(
            &self.ids,
            &self.verifier.pubkey(),
            &provider,
            job_id,
            &receipt.gpu_class,
            receipt.gpu_count,
            receipt.execution_duration_sec,
            receipt.scu_amount,
            result_hash,
            provider_signature,
        )?;

        self.send_signed(&[ix]).await
    }

    /// Build, sign, send, and confirm a `release_escrow` transaction.
    ///
    /// Also ensures the provider's associated token account for the mint
    /// exists by prepending an idempotent ATA creation instruction.
    pub async fn release_escrow(
        &self,
        job_uuid: uuid::Uuid,
        provider_pubkey: &str,
    ) -> Result<Signature> {
        let provider = Pubkey::from_str(provider_pubkey)
            .context("provider_pubkey is not a valid Solana pubkey")?;

        let job_id = job_id_from_uuid(&job_uuid);

        // Ensure provider ATA exists (no-op if already there).
        let provider_ata = get_associated_token_address(&provider, &self.mint);
        tracing::debug!("Provider ATA: {}", provider_ata);

        let create_ata_ix = create_associated_token_account_idempotent(
            &self.verifier.pubkey(),
            &provider,
            &self.mint,
            &spl_token::ID,
        );

        let release_ix = ix::release_escrow_ix(
            &self.ids,
            &self.verifier.pubkey(),
            &provider,
            &self.mint,
            job_id,
        )?;

        self.send_signed(&[create_ata_ix, release_ix]).await
    }

    async fn send_signed(
        &self,
        instructions: &[solana_sdk::instruction::Instruction],
    ) -> Result<Signature> {
        let blockhash = self
            .rpc
            .get_latest_blockhash()
            .await
            .context("get_latest_blockhash")?;

        let tx = Transaction::new_signed_with_payer(
            instructions,
            Some(&self.verifier.pubkey()),
            &[&self.verifier],
            blockhash,
        );

        let sig = self
            .rpc
            .send_and_confirm_transaction(&tx)
            .await
            .context("send_and_confirm_transaction")?;

        Ok(sig)
    }
}

fn load_keypair(path: &str) -> Result<Keypair> {
    let expanded = shellexpand::tilde(path).into_owned();
    read_keypair_file(&expanded)
        .map_err(|e| anyhow!("failed to read keypair {}: {}", expanded, e))
}

fn job_id_from_uuid(uuid: &uuid::Uuid) -> [u8; 32] {
    let mut out = [0u8; 32];
    out[..16].copy_from_slice(uuid.as_bytes());
    out
}

fn decode_fixed<const N: usize>(s: &str, field: &str) -> Result<[u8; N]> {
    let bytes = hex::decode(s).with_context(|| format!("{} not valid hex", field))?;
    if bytes.len() != N {
        return Err(anyhow!(
            "{} expected {} bytes, got {}",
            field,
            N,
            bytes.len()
        ));
    }
    let mut out = [0u8; N];
    out.copy_from_slice(&bytes);
    Ok(out)
}
