use anchor_lang::prelude::*;

pub mod instructions;
pub mod state;
pub mod errors;

use instructions::*;

declare_id!("7m465Af6QGcfmd6PgfxQkhUwTuWqLDBaiCS8mGgPDc12");

#[program]
pub mod provider_registry {
    use super::*;

    /// Register a new GPU provider on-chain.
    pub fn register_provider(
        ctx: Context<RegisterProvider>,
        name: String,
        gpu_class: String,
        gpu_count: u8,
        max_scu_per_epoch: u64,
    ) -> Result<()> {
        instructions::register_provider::handler(ctx, name, gpu_class, gpu_count, max_scu_per_epoch)
    }

    /// Update an existing provider's status or GPU configuration.
    pub fn update_provider(
        ctx: Context<UpdateProvider>,
        status: ProviderStatus,
        gpu_count: Option<u8>,
        max_scu_per_epoch: Option<u64>,
    ) -> Result<()> {
        instructions::update_provider::handler(ctx, status, gpu_count, max_scu_per_epoch)
    }
}
