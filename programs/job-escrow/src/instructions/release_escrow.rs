use anchor_lang::prelude::*;
use anchor_spl::token_interface::{
    Mint, TokenAccount, TokenInterface, TransferChecked, transfer_checked,
};
use crate::state::{EscrowAccount, EscrowStatus};
use crate::errors::EscrowError;

#[derive(Accounts)]
pub struct ReleaseEscrow<'info> {
    #[account(
        mut,
        seeds = [b"escrow", escrow.job_id.as_ref()],
        bump = escrow.bump,
    )]
    pub escrow: Account<'info, EscrowAccount>,

    #[account(
        mut,
        seeds = [b"vault", escrow.job_id.as_ref()],
        bump = escrow.vault_bump,
        token::mint = mint,
        token::authority = escrow,
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    pub mint: InterfaceAccount<'info, Mint>,

    /// Provider's token account to receive payment.
    #[account(
        mut,
        token::mint = mint,
    )]
    pub provider_token_account: InterfaceAccount<'info, TokenAccount>,

    /// The verifier/authority that approves the release.
    pub authority: Signer<'info>,

    pub token_program: Interface<'info, TokenInterface>,
}

pub fn handler(ctx: Context<ReleaseEscrow>) -> Result<()> {
    let escrow = &mut ctx.accounts.escrow;
    require!(escrow.status == EscrowStatus::Funded, EscrowError::NotFunded);

    let clock = Clock::get()?;
    escrow.status = EscrowStatus::Released;
    escrow.released_at = clock.unix_timestamp;

    // Transfer from vault to provider using escrow PDA as signer
    let job_id = escrow.job_id;
    let bump = escrow.bump;
    let signer_seeds: &[&[&[u8]]] = &[&[b"escrow", job_id.as_ref(), &[bump]]];

    let cpi_accounts = TransferChecked {
        from: ctx.accounts.vault.to_account_info(),
        to: ctx.accounts.provider_token_account.to_account_info(),
        authority: ctx.accounts.escrow.to_account_info(),
        mint: ctx.accounts.mint.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    transfer_checked(
        CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds),
        escrow.amount,
        ctx.accounts.mint.decimals,
    )?;

    msg!("Escrow released for job: {:?}", job_id);
    Ok(())
}
