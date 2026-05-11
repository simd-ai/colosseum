use anchor_lang::prelude::*;

/// On-chain state for a registered GPU compute provider.
#[account]
#[derive(InitSpace)]
pub struct ProviderAccount {
    /// The wallet authority that owns this provider.
    pub authority: Pubkey,
    /// Human-readable provider name.
    #[max_len(64)]
    pub name: String,
    /// GPU class identifier (e.g., "A100", "H100").
    #[max_len(16)]
    pub gpu_class: String,
    /// Number of GPUs available.
    pub gpu_count: u8,
    /// Maximum SCU (Standard Compute Units) capacity per epoch.
    pub max_scu_per_epoch: u64,
    /// Current provider status.
    pub status: ProviderStatus,
    /// Total jobs completed by this provider.
    pub total_jobs_completed: u64,
    /// Total SCU delivered across all jobs.
    pub total_scu_delivered: u64,
    /// Unix timestamp of registration.
    pub registered_at: i64,
    /// PDA bump seed.
    pub bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, InitSpace)]
pub enum ProviderStatus {
    Active,
    Inactive,
    Suspended,
}

impl Default for ProviderStatus {
    fn default() -> Self {
        ProviderStatus::Active
    }
}
