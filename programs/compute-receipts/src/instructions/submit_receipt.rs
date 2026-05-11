use anchor_lang::prelude::*;
use crate::state::ComputeReceipt;
use crate::errors::ReceiptError;

#[derive(Accounts)]
#[instruction(job_id: [u8; 32])]
pub struct SubmitReceipt<'info> {
    #[account(
        init,
        payer = verifier,
        space = 8 + ComputeReceipt::INIT_SPACE,
        seeds = [b"receipt", job_id.as_ref()],
        bump,
    )]
    pub receipt: Account<'info, ComputeReceipt>,

    /// CHECK: The provider pubkey, verified off-chain by the verification service.
    pub provider: UncheckedAccount<'info>,

    /// The verifier authority who signs to approve the receipt.
    #[account(mut)]
    pub verifier: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[allow(clippy::too_many_arguments)]
pub fn handler(
    ctx: Context<SubmitReceipt>,
    job_id: [u8; 32],
    gpu_class: String,
    gpu_count: u8,
    execution_duration_sec: u32,
    scu_amount: u64,
    result_hash: [u8; 32],
    provider_signature: [u8; 64],
) -> Result<()> {
    require!(gpu_class.len() <= 16, ReceiptError::GpuClassTooLong);
    require!(gpu_count >= 1, ReceiptError::InvalidGpuCount);
    require!(execution_duration_sec > 0, ReceiptError::InvalidDuration);
    require!(scu_amount > 0, ReceiptError::InvalidScuAmount);

    let clock = Clock::get()?;
    let receipt = &mut ctx.accounts.receipt;

    receipt.job_id = job_id;
    receipt.provider = ctx.accounts.provider.key();
    receipt.gpu_class = gpu_class;
    receipt.gpu_count = gpu_count;
    receipt.execution_duration_sec = execution_duration_sec;
    receipt.scu_amount = scu_amount;
    receipt.result_hash = result_hash;
    receipt.provider_signature = provider_signature;
    receipt.verified = true;
    receipt.verifier = ctx.accounts.verifier.key();
    receipt.submitted_at = clock.unix_timestamp;
    receipt.bump = ctx.bumps.receipt;

    msg!("Compute receipt submitted for job: {:?}", job_id);
    Ok(())
}
