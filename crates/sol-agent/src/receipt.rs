use ed25519_dalek::{Signer, SigningKey};
use uuid::Uuid;
use sol_common::ComputeReceiptData;

/// Build a compute receipt and sign it with the provider's key.
pub fn build_receipt(
    job_id: Uuid,
    provider_pubkey: &str,
    gpu_class: &str,
    gpu_count: u8,
    execution_duration_sec: u32,
    signing_key: &SigningKey,
) -> ComputeReceiptData {
    // Calculate SCU: gpu_count × duration × class_multiplier
    let class_multiplier: u64 = match gpu_class {
        "H100" => 10,
        "A100" => 8,
        "L40S" => 6,
        _      => 5,
    };
    let scu_amount = (gpu_count as u64) * (execution_duration_sec as u64) * class_multiplier;

    // Generate deterministic result hash
    let hash_input = format!("{}{}{}", job_id, provider_pubkey, execution_duration_sec);
    let result_hash = sol_common::sha256_hash(hash_input.as_bytes());
    let result_hash_hex = sol_common::to_hex(&result_hash);

    // Build canonical message and sign
    let message = sol_common::build_receipt_message(
        job_id.as_bytes(),
        provider_pubkey,
        gpu_class,
        gpu_count,
        execution_duration_sec,
        scu_amount,
        &result_hash,
    );
    let signature = signing_key.sign(&message);
    let signature_hex = sol_common::to_hex(&signature.to_bytes());

    let receipt = ComputeReceiptData {
        job_id,
        provider_pubkey: provider_pubkey.to_string(),
        gpu_class: gpu_class.to_string(),
        gpu_count,
        execution_duration_sec,
        scu_amount,
        result_hash: result_hash_hex,
        provider_signature: signature_hex,
    };

    tracing::info!(
        "📝 Receipt built: job={} scu={} duration={}s hash={}...",
        job_id,
        scu_amount,
        execution_duration_sec,
        &receipt.result_hash[..16],
    );

    receipt
}
