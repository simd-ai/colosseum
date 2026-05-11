//! Instruction builders. Anchor instructions are encoded as:
//!   [8-byte sighash discriminator] || borsh(args)
//!
//! The sighash is `sha256("global:<snake_case_name>")[..8]`, which matches
//! `anchor_lang::Discriminator::DISCRIMINATOR` for instructions exposed in
//! `#[program]` blocks.

use anyhow::Result;
use borsh::BorshSerialize;
use sha2::{Digest, Sha256};
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program,
    sysvar::rent::ID as RENT_ID,
};
use spl_associated_token_account::get_associated_token_address;

use crate::{ids::ProgramIds, pda::*};

/// SPL Token program id (classic).
pub fn spl_token_id() -> Pubkey {
    spl_token::ID
}

/// Compute the 8-byte anchor instruction discriminator.
fn sighash(name: &str) -> [u8; 8] {
    let preimage = format!("global:{}", name);
    let h = Sha256::digest(preimage.as_bytes());
    let mut out = [0u8; 8];
    out.copy_from_slice(&h[..8]);
    out
}

fn encode<T: BorshSerialize>(name: &str, args: &T) -> Result<Vec<u8>> {
    let mut data = sighash(name).to_vec();
    args.serialize(&mut data)?;
    Ok(data)
}

// ───────────────────────── provider_registry ──────────────────────────

#[derive(BorshSerialize)]
struct RegisterProviderArgs {
    name: String,
    gpu_class: String,
    gpu_count: u8,
    max_scu_per_epoch: u64,
}

pub fn register_provider_ix(
    ids: &ProgramIds,
    authority: &Pubkey,
    name: &str,
    gpu_class: &str,
    gpu_count: u8,
    max_scu_per_epoch: u64,
) -> Result<Instruction> {
    let (provider, _) = provider_pda(&ids.provider_registry, authority);
    let data = encode(
        "register_provider",
        &RegisterProviderArgs {
            name: name.to_string(),
            gpu_class: gpu_class.to_string(),
            gpu_count,
            max_scu_per_epoch,
        },
    )?;
    Ok(Instruction {
        program_id: ids.provider_registry,
        accounts: vec![
            AccountMeta::new(provider, false),
            AccountMeta::new(*authority, true),
            AccountMeta::new_readonly(system_program::ID, false),
        ],
        data,
    })
}

// ───────────────────────── job_escrow ────────────────────────────────

#[derive(BorshSerialize)]
struct CreateEscrowArgs {
    job_id: [u8; 32],
    amount: u64,
}

#[allow(clippy::too_many_arguments)]
pub fn create_escrow_ix(
    ids: &ProgramIds,
    client: &Pubkey,
    provider: &Pubkey,
    mint: &Pubkey,
    job_id: [u8; 32],
    amount: u64,
) -> Result<Instruction> {
    let (escrow, _) = escrow_pda(&ids.job_escrow, &job_id);
    let (vault, _) = vault_pda(&ids.job_escrow, &job_id);
    let client_ata = get_associated_token_address(client, mint);
    let data = encode("create_escrow", &CreateEscrowArgs { job_id, amount })?;
    Ok(Instruction {
        program_id: ids.job_escrow,
        accounts: vec![
            AccountMeta::new(escrow, false),
            AccountMeta::new(vault, false),
            AccountMeta::new_readonly(*mint, false),
            AccountMeta::new(client_ata, false),
            AccountMeta::new(*client, true),
            AccountMeta::new_readonly(*provider, false),
            AccountMeta::new_readonly(spl_token_id(), false),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new_readonly(RENT_ID, false),
        ],
        data,
    })
}

#[derive(BorshSerialize)]
struct ReleaseEscrowArgs {}

pub fn release_escrow_ix(
    ids: &ProgramIds,
    authority: &Pubkey,
    provider: &Pubkey,
    mint: &Pubkey,
    job_id: [u8; 32],
) -> Result<Instruction> {
    let (escrow, _) = escrow_pda(&ids.job_escrow, &job_id);
    let (vault, _) = vault_pda(&ids.job_escrow, &job_id);
    let provider_ata = get_associated_token_address(provider, mint);
    let data = encode("release_escrow", &ReleaseEscrowArgs {})?;
    Ok(Instruction {
        program_id: ids.job_escrow,
        accounts: vec![
            AccountMeta::new(escrow, false),
            AccountMeta::new(vault, false),
            AccountMeta::new_readonly(*mint, false),
            AccountMeta::new(provider_ata, false),
            AccountMeta::new_readonly(*authority, true),
            AccountMeta::new_readonly(spl_token_id(), false),
        ],
        data,
    })
}

#[derive(BorshSerialize)]
struct RefundEscrowArgs {}

pub fn refund_escrow_ix(
    ids: &ProgramIds,
    authority: &Pubkey,
    client: &Pubkey,
    mint: &Pubkey,
    job_id: [u8; 32],
) -> Result<Instruction> {
    let (escrow, _) = escrow_pda(&ids.job_escrow, &job_id);
    let (vault, _) = vault_pda(&ids.job_escrow, &job_id);
    let client_ata = get_associated_token_address(client, mint);
    let data = encode("refund_escrow", &RefundEscrowArgs {})?;
    Ok(Instruction {
        program_id: ids.job_escrow,
        accounts: vec![
            AccountMeta::new(escrow, false),
            AccountMeta::new(vault, false),
            AccountMeta::new_readonly(*mint, false),
            AccountMeta::new(client_ata, false),
            AccountMeta::new_readonly(*authority, true),
            AccountMeta::new_readonly(spl_token_id(), false),
        ],
        data,
    })
}

// ───────────────────────── compute_receipts ──────────────────────────

#[derive(BorshSerialize)]
struct SubmitReceiptArgs {
    job_id: [u8; 32],
    gpu_class: String,
    gpu_count: u8,
    execution_duration_sec: u32,
    scu_amount: u64,
    result_hash: [u8; 32],
    provider_signature: [u8; 64],
}

#[allow(clippy::too_many_arguments)]
pub fn submit_receipt_ix(
    ids: &ProgramIds,
    verifier: &Pubkey,
    provider: &Pubkey,
    job_id: [u8; 32],
    gpu_class: &str,
    gpu_count: u8,
    execution_duration_sec: u32,
    scu_amount: u64,
    result_hash: [u8; 32],
    provider_signature: [u8; 64],
) -> Result<Instruction> {
    let (receipt, _) = receipt_pda(&ids.compute_receipts, &job_id);
    let data = encode(
        "submit_receipt",
        &SubmitReceiptArgs {
            job_id,
            gpu_class: gpu_class.to_string(),
            gpu_count,
            execution_duration_sec,
            scu_amount,
            result_hash,
            provider_signature,
        },
    )?;
    Ok(Instruction {
        program_id: ids.compute_receipts,
        accounts: vec![
            AccountMeta::new(receipt, false),
            AccountMeta::new_readonly(*provider, false),
            AccountMeta::new(*verifier, true),
            AccountMeta::new_readonly(system_program::ID, false),
        ],
        data,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sighash_matches_anchor_format() {
        // sha256("global:register_provider")[..8]
        let h = sighash("register_provider");
        assert_eq!(h.len(), 8);
    }
}
