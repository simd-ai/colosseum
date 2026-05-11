use anyhow::Result;
use sol_common::TxRequest;

/// Submit a Solana transaction based on the TX request type.
///
/// In demo mode, this generates a mock transaction signature
/// instead of actually submitting to Solana. For production,
/// this would use `solana-client` to build, sign, and send
/// the actual Anchor instruction transactions.
pub async fn submit_transaction(
    tx_request: &TxRequest,
    _config: &sol_common::AppConfig,
) -> Result<String> {
    tracing::debug!("Submitting TX: {:?} for {}", tx_request.tx_type, tx_request.reference_id);

    // Simulate transaction submission delay
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // Generate a deterministic mock signature based on the request
    let sig_input = format!("{}:{}", tx_request.tx_type, tx_request.reference_id);
    let hash = sol_common::sha256_hash(sig_input.as_bytes());
    let mock_signature = bs58::encode(&hash).into_string();

    // In production, this would:
    // 1. Build the appropriate Anchor instruction (register_provider, create_escrow, etc.)
    // 2. Create the transaction with recent blockhash
    // 3. Sign with the authority keypair
    // 4. Send via RPC client
    // 5. Return the actual signature

    tracing::debug!("Mock TX signature: {}", mock_signature);
    Ok(mock_signature)
}
