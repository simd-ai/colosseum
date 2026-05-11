use anchor_lang::prelude::*;
use crate::state::{ProviderAccount, ProviderStatus};
use crate::errors::RegistryError;

#[derive(Accounts)]
pub struct RegisterProvider<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + ProviderAccount::INIT_SPACE,
        seeds = [b"provider", authority.key().as_ref()],
        bump,
    )]
    pub provider: Account<'info, ProviderAccount>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<RegisterProvider>,
    name: String,
    gpu_class: String,
    gpu_count: u8,
    max_scu_per_epoch: u64,
) -> Result<()> {
    require!(name.len() <= 64, RegistryError::NameTooLong);
    require!(gpu_class.len() <= 16, RegistryError::GpuClassTooLong);
    require!(gpu_count >= 1, RegistryError::InvalidGpuCount);
    require!(max_scu_per_epoch > 0, RegistryError::InvalidMaxScu);

    let provider = &mut ctx.accounts.provider;
    let clock = Clock::get()?;

    provider.authority = ctx.accounts.authority.key();
    provider.name = name;
    provider.gpu_class = gpu_class;
    provider.gpu_count = gpu_count;
    provider.max_scu_per_epoch = max_scu_per_epoch;
    provider.status = ProviderStatus::Active;
    provider.total_jobs_completed = 0;
    provider.total_scu_delivered = 0;
    provider.registered_at = clock.unix_timestamp;
    provider.bump = ctx.bumps.provider;

    msg!("Provider registered: {}", provider.authority);
    Ok(())
}
