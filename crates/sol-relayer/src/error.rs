#[derive(Debug, thiserror::Error)]
pub enum RelayerError {
    #[error("Transaction submission failed: {0}")]
    SubmissionFailed(String),
    #[error("Transaction confirmation timeout: {0}")]
    ConfirmationTimeout(String),
    #[error("Max retries exceeded for transaction: {0}")]
    MaxRetriesExceeded(String),
}
