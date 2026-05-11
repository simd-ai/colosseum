//! PDA derivation matching the seeds in our Anchor programs.

use solana_sdk::pubkey::Pubkey;

/// `[b"provider", authority]` under provider_registry program.
pub fn provider_pda(program_id: &Pubkey, authority: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"provider", authority.as_ref()], program_id)
}

/// `[b"escrow", job_id]` under job_escrow program.
pub fn escrow_pda(program_id: &Pubkey, job_id: &[u8; 32]) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"escrow", job_id.as_ref()], program_id)
}

/// `[b"vault", job_id]` under job_escrow program — token account for the escrow.
pub fn vault_pda(program_id: &Pubkey, job_id: &[u8; 32]) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"vault", job_id.as_ref()], program_id)
}

/// `[b"receipt", job_id]` under compute_receipts program.
pub fn receipt_pda(program_id: &Pubkey, job_id: &[u8; 32]) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"receipt", job_id.as_ref()], program_id)
}

/// Convert a uuid::Uuid-equivalent 16 bytes into the 32-byte job_id we use on-chain.
/// We pad with zeros so the same value is reproducible from off-chain code.
pub fn job_id_from_uuid_bytes(uuid_bytes: &[u8; 16]) -> [u8; 32] {
    let mut out = [0u8; 32];
    out[..16].copy_from_slice(uuid_bytes);
    out
}
