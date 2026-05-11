use anchor_lang::prelude::*;
use crate::state::{ProviderAccount, ProviderStatus};
use crate::errors::RegistryError;

#[derive(Accounts)]
pub struct UpdateProvider<'info> {
    #[account(
        mut,
        seeds = [b"provider", authority.key().as_ref()],
        bump = provider.bump,
        has_one = authority @ RegistryError::Unauthorized,
    )]
    pub provider: Account<'info, ProviderAccount>,

    pub authority: Signer<'info>,
}

pub fn handler(
    ctx: Context<UpdateProvider>,
    status: ProviderStatus,
    gpu_count: Option<u8>,
    max_scu_per_epoch: Option<u64>,
) -> Result<()> {
    let provider = &mut ctx.accounts.provider;

    provider.status = status;

    if let Some(count) = gpu_count {
        require!(count >= 1, RegistryError::InvalidGpuCount);
        provider.gpu_count = count;
    }

    if let Some(max_scu) = max_scu_per_epoch {
        require!(max_scu > 0, RegistryError::InvalidMaxScu);
        provider.max_scu_per_epoch = max_scu;
    }

    msg!("Provider updated: {}", provider.authority);
    Ok(())
}
