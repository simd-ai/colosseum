//! Build and sign a ComputeReceipt using the agent's Solana keypair.
//!
//! Solana keypairs are Ed25519, so the signature this produces is exactly the
//! same bytes the verifier (and the on-chain `submit_receipt` program) expect.

use solana_sdk::signature::{Keypair, Signer};
use uuid::Uuid;

use sol_common::ComputeReceiptData;

pub fn build_receipt(
    job_id: Uuid,
    provider_pubkey: &str,
    gpu_class: &str,
    gpu_count: u8,
    execution_duration_sec: u32,
    keypair: &Keypair,
) -> ComputeReceiptData {
    let class_multiplier: u64 = match gpu_class {
        "H100" => 10,
        "A100" => 8,
        "L40S" => 6,
        _ => 5,
    };
    let scu_amount =
        (gpu_count as u64) * (execution_duration_sec as u64) * class_multiplier;

    // Deterministic result hash so verifier+chain agree.
    let hash_input = format!("{}{}{}", job_id, provider_pubkey, execution_duration_sec);
    let result_hash = sol_common::sha256_hash(hash_input.as_bytes());
    let result_hash_hex = sol_common::to_hex(&result_hash);

    let message = sol_common::build_receipt_message(
        job_id.as_bytes(),
        provider_pubkey,
        gpu_class,
        gpu_count,
        execution_duration_sec,
        scu_amount,
        &result_hash,
    );
    let signature = keypair.sign_message(&message);
    let signature_hex = sol_common::to_hex(signature.as_ref());

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
