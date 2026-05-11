use anyhow::Result;

/// Confirmation status for a Solana transaction.
#[derive(Debug, Clone, PartialEq)]
pub enum ConfirmationStatus {
    Pending,
    Confirmed,
    Finalized,
    Failed(String),
}

/// Poll for transaction confirmation.
///
/// In demo mode, this immediately returns Confirmed.
/// In production, this would poll `getSignatureStatuses` RPC.
pub async fn wait_for_confirmation(
    _tx_hash: &str,
    _rpc_url: &str,
    _max_retries: u32,
) -> Result<ConfirmationStatus> {
    // Simulate confirmation delay
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    // In production:
    // loop {
    //     let status = rpc_client.get_signature_statuses(&[signature]).await?;
    //     match status.value[0] {
    //         Some(TransactionStatus { confirmation_status: Some(Confirmed), .. }) => return Ok(Confirmed),
    //         Some(TransactionStatus { err: Some(e), .. }) => return Ok(Failed(e.to_string())),
    //         _ => tokio::time::sleep(Duration::from_millis(500)).await,
    //     }
    //     retries += 1;
    //     if retries >= max_retries { return Ok(Pending); }
    // }

    Ok(ConfirmationStatus::Confirmed)
}
