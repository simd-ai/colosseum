use anchor_lang::prelude::*;

pub mod instructions;
pub mod state;
pub mod errors;

use instructions::*;

declare_id!("5yCy1zqLYQh6u9NjCFU9mQSfR2RinzUA9TdLSkj5TYeu");

#[program]
pub mod job_escrow {
    use super::*;

    /// Create escrow: client deposits SPL tokens for a compute job.
    pub fn create_escrow(
        ctx: Context<CreateEscrow>,
        job_id: [u8; 32],
        amount: u64,
    ) -> Result<()> {
        instructions::create_escrow::handler(ctx, job_id, amount)
    }

    /// Release escrow: verifier authority releases funds to provider after verified receipt.
    pub fn release_escrow(ctx: Context<ReleaseEscrow>) -> Result<()> {
        instructions::release_escrow::handler(ctx)
    }

    /// Refund escrow: return funds to client on job failure or cancellation.
    pub fn refund_escrow(ctx: Context<RefundEscrow>) -> Result<()> {
        instructions::refund_escrow::handler(ctx)
    }
}
