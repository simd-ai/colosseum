use anchor_lang::prelude::*;
use anchor_spl::token_interface::{
    Mint, TokenAccount, TokenInterface, TransferChecked, transfer_checked,
};
use crate::state::{EscrowAccount, EscrowStatus};
use crate::errors::EscrowError;

#[derive(Accounts)]
#[instruction(job_id: [u8; 32])]
pub struct CreateEscrow<'info> {
    #[account(
        init,
        payer = client,
        space = 8 + EscrowAccount::INIT_SPACE,
        seeds = [b"escrow", job_id.as_ref()],
        bump,
    )]
    pub escrow: Account<'info, EscrowAccount>,

    #[account(
        init,
        payer = client,
        token::mint = mint,
        token::authority = escrow,
        seeds = [b"vault", job_id.as_ref()],
        bump,
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        token::mint = mint,
        token::authority = client,
    )]
    pub client_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(mut)]
    pub client: Signer<'info>,

    /// CHECK: Provider pubkey, validated off-chain by the scheduler.
    pub provider: UncheckedAccount<'info>,

    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<CreateEscrow>,
    job_id: [u8; 32],
    amount: u64,
) -> Result<()> {
    require!(amount > 0, EscrowError::InvalidAmount);

    let clock = Clock::get()?;
    let escrow = &mut ctx.accounts.escrow;

    escrow.job_id = job_id;
    escrow.client = ctx.accounts.client.key();
    escrow.provider = ctx.accounts.provider.key();
    escrow.mint = ctx.accounts.mint.key();
    escrow.escrow_vault = ctx.accounts.vault.key();
    escrow.amount = amount;
    escrow.status = EscrowStatus::Funded;
    escrow.created_at = clock.unix_timestamp;
    escrow.released_at = 0;
    escrow.bump = ctx.bumps.escrow;
    escrow.vault_bump = ctx.bumps.vault;

    // Transfer tokens from client to vault
    let cpi_accounts = TransferChecked {
        from: ctx.accounts.client_token_account.to_account_info(),
        to: ctx.accounts.vault.to_account_info(),
        authority: ctx.accounts.client.to_account_info(),
        mint: ctx.accounts.mint.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    transfer_checked(
        CpiContext::new(cpi_program, cpi_accounts),
        amount,
        ctx.accounts.mint.decimals,
    )?;

    msg!("Escrow created for job: {:?}", job_id);
    Ok(())
}
