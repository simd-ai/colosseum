use ed25519_dalek::{Signer, SigningKey, Verifier, VerifyingKey, Signature};
use sha2::{Sha256, Digest};
use rand::RngCore;

/// Generate a new Ed25519 keypair using OsRng for entropy.
///
/// ed25519-dalek 2.2 removed the rand_core impl-based `generate(&mut rng)`
/// helper, so we fill 32 random bytes directly and feed them in.
pub fn generate_keypair() -> (SigningKey, VerifyingKey) {
    let mut secret = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut secret);
    let signing_key = SigningKey::from_bytes(&secret);
    let verifying_key = signing_key.verifying_key();
    (signing_key, verifying_key)
}

/// Sign a message with an Ed25519 signing key.
pub fn sign_message(signing_key: &SigningKey, message: &[u8]) -> Signature {
    signing_key.sign(message)
}

/// Verify an Ed25519 signature.
pub fn verify_signature(
    verifying_key: &VerifyingKey,
    message: &[u8],
    signature: &Signature,
) -> bool {
    verifying_key.verify(message, signature).is_ok()
}

/// Hash data with SHA-256 and return the 32-byte digest.
pub fn sha256_hash(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&result);
    hash
}

/// Build the canonical message for receipt signing.
/// Format: job_id || provider_pubkey || gpu_class || gpu_count || duration || scu_amount || result_hash
pub fn build_receipt_message(
    job_id: &[u8],
    provider_pubkey: &str,
    gpu_class: &str,
    gpu_count: u8,
    execution_duration_sec: u32,
    scu_amount: u64,
    result_hash: &[u8],
) -> Vec<u8> {
    let mut msg = Vec::new();
    msg.extend_from_slice(job_id);
    msg.extend_from_slice(provider_pubkey.as_bytes());
    msg.extend_from_slice(gpu_class.as_bytes());
    msg.push(gpu_count);
    msg.extend_from_slice(&execution_duration_sec.to_le_bytes());
    msg.extend_from_slice(&scu_amount.to_le_bytes());
    msg.extend_from_slice(result_hash);
    msg
}

/// Encode bytes to hex string.
pub fn to_hex(bytes: &[u8]) -> String {
    hex::encode(bytes)
}

/// Decode hex string to bytes.
pub fn from_hex(s: &str) -> Result<Vec<u8>, hex::FromHexError> {
    hex::decode(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_verify_roundtrip() {
        let (signing_key, verifying_key) = generate_keypair();
        let message = b"test message for SolGrid";
        let signature = sign_message(&signing_key, message);
        assert!(verify_signature(&verifying_key, message, &signature));
    }

    #[test]
    fn test_sha256_deterministic() {
        let data = b"deterministic hash test";
        let hash1 = sha256_hash(data);
        let hash2 = sha256_hash(data);
        assert_eq!(hash1, hash2);
    }
}
