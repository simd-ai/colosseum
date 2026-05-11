use anchor_lang::prelude::*;

#[error_code]
pub enum ReceiptError {
    #[msg("GPU class exceeds maximum length of 16 characters")]
    GpuClassTooLong,
    #[msg("GPU count must be at least 1")]
    InvalidGpuCount,
    #[msg("Execution duration must be greater than zero")]
    InvalidDuration,
    #[msg("SCU amount must be greater than zero")]
    InvalidScuAmount,
    #[msg("Receipt has already been submitted for this job")]
    DuplicateReceipt,
}
