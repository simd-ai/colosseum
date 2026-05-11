use anchor_lang::prelude::*;
use anchor_spl::token_interface::{
    Mint, TokenAccount, TokenInterface, TransferChecked, transfer_checked,
};
use crate::state::{EscrowAccount, EscrowStatus};
use crate::errors::EscrowError;

#[derive(Accounts)]
pub struct RefundEscrow<'info> {
    #[account(
        mut,
        seeds = [b"escrow", escrow.job_id.as_ref()],
        bump = escrow.bump,
        has_one = client @ EscrowError::Unauthorized,
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

    /// Client's token account to receive refund.
    #[account(
        mut,
        token::mint = mint,
        token::authority = client,
    )]
    pub client_token_account: InterfaceAccount<'info, TokenAccount>,

    pub client: Signer<'info>,

    pub token_program: Interface<'info, TokenInterface>,
}

pub fn handler(ctx: Context<RefundEscrow>) -> Result<()> {
    require!(
        ctx.accounts.escrow.status == EscrowStatus::Funded,
        EscrowError::NotFunded
    );

    let job_id = ctx.accounts.escrow.job_id;
    let bump = ctx.accounts.escrow.bump;
    let amount = ctx.accounts.escrow.amount;
    let decimals = ctx.accounts.mint.decimals;

    let escrow_ai = ctx.accounts.escrow.to_account_info();
    let vault_ai = ctx.accounts.vault.to_account_info();
    let client_ata_ai = ctx.accounts.client_token_account.to_account_info();
    let mint_ai = ctx.accounts.mint.to_account_info();
    let token_program_ai = ctx.accounts.token_program.to_account_info();

    let signer_seeds: &[&[&[u8]]] = &[&[b"escrow", job_id.as_ref(), &[bump]]];
    transfer_checked(
        CpiContext::new_with_signer(
            token_program_ai,
            TransferChecked {
                from: vault_ai,
                to: client_ata_ai,
                authority: escrow_ai,
                mint: mint_ai,
            },
            signer_seeds,
        ),
        amount,
        decimals,
    )?;

    ctx.accounts.escrow.status = EscrowStatus::Refunded;

    msg!("Escrow refunded for job: {:?}", job_id);
    Ok(())
}
