use anchor_lang::prelude::*;

#[error_code]
pub enum RegistryError {
    #[msg("Provider name exceeds maximum length of 64 characters")]
    NameTooLong,
    #[msg("GPU class exceeds maximum length of 16 characters")]
    GpuClassTooLong,
    #[msg("GPU count must be at least 1")]
    InvalidGpuCount,
    #[msg("Max SCU per epoch must be greater than zero")]
    InvalidMaxScu,
    #[msg("Provider is not active")]
    ProviderNotActive,
    #[msg("Unauthorized: signer is not the provider authority")]
    Unauthorized,
}
