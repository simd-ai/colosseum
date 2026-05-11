//! Program IDs. These are overridden at runtime from env vars by the relayer / CLI;
//! the constants here exist so the on-chain helpers compile standalone.

use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

/// Default placeholder IDs — only valid until `anchor deploy` produces real ones.
pub const PROVIDER_REGISTRY_ID_STR: &str = "Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS";
pub const JOB_ESCROW_ID_STR: &str = "HmbTLCmaGtYhSsT3D2RzKFkN3CKqZbHN4KxrFEhMccgH";
pub const COMPUTE_RECEIPTS_ID_STR: &str = "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM";

pub fn provider_registry_id() -> Pubkey {
    Pubkey::from_str(PROVIDER_REGISTRY_ID_STR).expect("valid pubkey")
}

pub fn job_escrow_id() -> Pubkey {
    Pubkey::from_str(JOB_ESCROW_ID_STR).expect("valid pubkey")
}

pub fn compute_receipts_id() -> Pubkey {
    Pubkey::from_str(COMPUTE_RECEIPTS_ID_STR).expect("valid pubkey")
}

/// Bundle of program IDs resolved at runtime.
#[derive(Debug, Clone, Copy)]
pub struct ProgramIds {
    pub provider_registry: Pubkey,
    pub job_escrow: Pubkey,
    pub compute_receipts: Pubkey,
}

impl Default for ProgramIds {
    fn default() -> Self {
        Self {
            provider_registry: provider_registry_id(),
            job_escrow: job_escrow_id(),
            compute_receipts: compute_receipts_id(),
        }
    }
}
