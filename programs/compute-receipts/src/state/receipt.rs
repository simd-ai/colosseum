use anchor_lang::prelude::*;

/// On-chain compute receipt proving work was performed.
#[account]
#[derive(InitSpace)]
pub struct ComputeReceipt {
    /// Unique job identifier.
    pub job_id: [u8; 32],
    /// Provider who performed the compute.
    pub provider: Pubkey,
    /// GPU class used (e.g., "A100").
    #[max_len(16)]
    pub gpu_class: String,
    /// Number of GPUs used.
    pub gpu_count: u8,
    /// Duration of compute execution in seconds.
    pub execution_duration_sec: u32,
    /// Standard Compute Units consumed.
    pub scu_amount: u64,
    /// SHA-256 hash of the compute result.
    pub result_hash: [u8; 32],
    /// Provider's Ed25519 signature over receipt fields.
    pub provider_signature: [u8; 64],
    /// Whether the receipt has been verified.
    pub verified: bool,
    /// The verifier authority who approved the receipt.
    pub verifier: Pubkey,
    /// Unix timestamp of submission.
    pub submitted_at: i64,
    /// PDA bump seed.
    pub bump: u8,
}
