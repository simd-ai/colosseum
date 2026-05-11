use anchor_lang::prelude::*;

pub mod instructions;
pub mod state;
pub mod errors;

use instructions::*;

declare_id!("9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM");

#[program]
pub mod compute_receipts {
    use super::*;

    /// Submit a verified compute receipt on-chain.
    pub fn submit_receipt(
        ctx: Context<SubmitReceipt>,
        job_id: [u8; 32],
        gpu_class: String,
        gpu_count: u8,
        execution_duration_sec: u32,
        scu_amount: u64,
        result_hash: [u8; 32],
        provider_signature: [u8; 64],
    ) -> Result<()> {
        instructions::submit_receipt::handler(
            ctx,
            job_id,
            gpu_class,
            gpu_count,
            execution_duration_sec,
            scu_amount,
            result_hash,
            provider_signature,
        )
    }
}
