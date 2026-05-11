use anchor_lang::prelude::*;

#[error_code]
pub enum EscrowError {
    #[msg("Escrow amount must be greater than zero")]
    InvalidAmount,
    #[msg("Escrow is not in Funded status")]
    NotFunded,
    #[msg("Escrow has already been released")]
    AlreadyReleased,
    #[msg("Escrow has already been refunded")]
    AlreadyRefunded,
    #[msg("Unauthorized: signer is not the escrow authority")]
    Unauthorized,
    #[msg("Invalid provider for this escrow")]
    InvalidProvider,
}
