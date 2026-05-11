use anchor_lang::prelude::*;

/// On-chain escrow account holding SPL tokens for a compute job.
#[account]
#[derive(InitSpace)]
pub struct EscrowAccount {
    /// Unique job identifier (UUID as 32 bytes).
    pub job_id: [u8; 32],
    /// The client who created and funded the escrow.
    pub client: Pubkey,
    /// The assigned GPU compute provider.
    pub provider: Pubkey,
    /// SPL token mint used for payment.
    pub mint: Pubkey,
    /// Token account (vault) holding escrowed funds.
    pub escrow_vault: Pubkey,
    /// Amount of tokens escrowed.
    pub amount: u64,
    /// Current escrow status.
    pub status: EscrowStatus,
    /// Unix timestamp of creation.
    pub created_at: i64,
    /// Unix timestamp of release (0 if not yet released).
    pub released_at: i64,
    /// PDA bump seed.
    pub bump: u8,
    /// Vault PDA bump seed.
    pub vault_bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, InitSpace)]
pub enum EscrowStatus {
    Funded,
    Released,
    Refunded,
}

impl Default for EscrowStatus {
    fn default() -> Self {
        EscrowStatus::Funded
    }
}
