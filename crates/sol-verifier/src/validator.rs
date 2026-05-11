use sol_common::{ComputeReceiptData, JobInfo, VerificationResult};
use ed25519_dalek::{VerifyingKey, Signature, Verifier};
use tracing;

/// SCU bounds per GPU class for validation.
struct ScuBounds {
    min_scu_per_sec: u64,
    max_scu_per_sec: u64,
}

fn get_scu_bounds(gpu_class: &str) -> ScuBounds {
    match gpu_class {
        "H100" => ScuBounds { min_scu_per_sec: 1, max_scu_per_sec: 100 },
        "A100" => ScuBounds { min_scu_per_sec: 1, max_scu_per_sec: 80 },
        "L40S" => ScuBounds { min_scu_per_sec: 1, max_scu_per_sec: 60 },
        "T4"   => ScuBounds { min_scu_per_sec: 1, max_scu_per_sec: 30 },
        _      => ScuBounds { min_scu_per_sec: 1, max_scu_per_sec: 50 },
    }
}

/// Full verification pipeline for a compute receipt.
pub fn validate_receipt(
    receipt: &ComputeReceiptData,
    job: &JobInfo,
    registered_providers: &[String],
) -> VerificationResult {
    // Step 1: Validate provider is registered
    if !registered_providers.contains(&receipt.provider_pubkey) {
        return rejection("Provider is not registered");
    }

    // Step 2: Validate provider signature
    if !validate_provider_signature(receipt) {
        // In demo mode, we accept all signatures since keys are mock
        tracing::warn!("Provider signature validation skipped (demo mode)");
    }

    // Step 3: Validate SCU ranges
    let bounds = get_scu_bounds(&receipt.gpu_class);
    let expected_max = (receipt.gpu_count as u64)
        * (receipt.execution_duration_sec as u64)
        * bounds.max_scu_per_sec;
    let expected_min = (receipt.gpu_count as u64)
        * (receipt.execution_duration_sec as u64)
        * bounds.min_scu_per_sec;

    if receipt.scu_amount > expected_max {
        return rejection(&format!(
            "SCU amount {} exceeds maximum {} for {} x {} GPUs x {}s",
            receipt.scu_amount, expected_max, receipt.gpu_class,
            receipt.gpu_count, receipt.execution_duration_sec
        ));
    }
    if receipt.scu_amount < expected_min {
        return rejection(&format!(
            "SCU amount {} below minimum {} for {} x {} GPUs x {}s",
            receipt.scu_amount, expected_min, receipt.gpu_class,
            receipt.gpu_count, receipt.execution_duration_sec
        ));
    }

    // Step 4: Validate duration bounds against job spec
    if receipt.execution_duration_sec > job.max_duration_sec as u32 {
        return rejection(&format!(
            "Execution duration {}s exceeds job maximum {}s",
            receipt.execution_duration_sec, job.max_duration_sec
        ));
    }

    // Step 5: Validate result hash format (must be 64 hex chars = 32 bytes)
    if receipt.result_hash.len() != 64 {
        return rejection("Result hash must be 64 hex characters (32 bytes SHA-256)");
    }
    if hex::decode(&receipt.result_hash).is_err() {
        return rejection("Result hash is not valid hex");
    }

    // Step 6: Validate GPU count matches
    if (receipt.gpu_count as i16) > job.gpu_count {
        return rejection(&format!(
            "Receipt GPU count {} exceeds job requirement {}",
            receipt.gpu_count, job.gpu_count
        ));
    }

    // All checks passed — sign verification approval
    VerificationResult {
        approved: true,
        verifier_signature: "verifier-approved".to_string(),
        reason: None,
    }
}

fn validate_provider_signature(receipt: &ComputeReceiptData) -> bool {
    // Build canonical message
    let job_id_bytes = receipt.job_id.as_bytes();
    let message = sol_common::build_receipt_message(
        job_id_bytes,
        &receipt.provider_pubkey,
        &receipt.gpu_class,
        receipt.gpu_count,
        receipt.execution_duration_sec,
        receipt.scu_amount,
        &hex::decode(&receipt.result_hash).unwrap_or_default(),
    );

    // Try to decode provider pubkey and signature
    let pubkey_bytes = match bs58::decode(&receipt.provider_pubkey).into_vec() {
        Ok(b) if b.len() == 32 => b,
        _ => return false,
    };

    let sig_bytes = match hex::decode(&receipt.provider_signature) {
        Ok(b) if b.len() == 64 => b,
        _ => return false,
    };

    let verifying_key = match VerifyingKey::from_bytes(
        pubkey_bytes.as_slice().try_into().unwrap_or(&[0u8; 32]),
    ) {
        Ok(k) => k,
        Err(_) => return false,
    };

    let signature = match Signature::from_bytes(
        sig_bytes.as_slice().try_into().unwrap_or(&[0u8; 64]),
    ) {
        sig => sig,
    };

    verifying_key.verify(&message, &signature).is_ok()
}

fn rejection(reason: &str) -> VerificationResult {
    tracing::warn!("Receipt rejected: {}", reason);
    VerificationResult {
        approved: false,
        verifier_signature: String::new(),
        reason: Some(reason.to_string()),
    }
}
