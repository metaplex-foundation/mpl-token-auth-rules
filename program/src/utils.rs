//! Utilities for the program
use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    sysvar::Sysvar,
};

use crate::{error::RuleSetError, payload::ProofInfo};

/// Create account almost from scratch, lifted from
/// <https://github.com/solana-labs/solana-program-library/tree/master/associated-token-account/program/src/processor.rs#L51-L98>
#[inline(always)]
pub fn create_or_allocate_account_raw<'a>(
    program_id: Pubkey,
    new_account_info: &AccountInfo<'a>,
    system_program_info: &AccountInfo<'a>,
    payer_info: &AccountInfo<'a>,
    size: usize,
    signer_seeds: &[&[u8]],
) -> ProgramResult {
    let rent = &Rent::get()?;
    let required_lamports = rent
        .minimum_balance(size)
        .max(1)
        .saturating_sub(new_account_info.lamports());

    if required_lamports > 0 {
        msg!("Transfer {} lamports to the new account", required_lamports);
        invoke(
            &system_instruction::transfer(payer_info.key, new_account_info.key, required_lamports),
            &[
                payer_info.clone(),
                new_account_info.clone(),
                system_program_info.clone(),
            ],
        )?;
    }

    let accounts = &[new_account_info.clone(), system_program_info.clone()];

    msg!("Allocate space for the account");
    invoke_signed(
        &system_instruction::allocate(new_account_info.key, size.try_into().unwrap()),
        accounts,
        &[signer_seeds],
    )?;

    msg!("Assign the account to the owning program");
    invoke_signed(
        &system_instruction::assign(new_account_info.key, &program_id),
        accounts,
        &[signer_seeds],
    )?;

    Ok(())
}

/// Resize an account using realloc, lifted from Solana Cookbook.
#[inline(always)]
pub fn resize_or_reallocate_account_raw<'a>(
    target_account: &AccountInfo<'a>,
    funding_account: &AccountInfo<'a>,
    system_program: &AccountInfo<'a>,
    new_size: usize,
) -> ProgramResult {
    let rent = Rent::get()?;
    let new_minimum_balance = rent.minimum_balance(new_size);

    let lamports_diff = new_minimum_balance.saturating_sub(target_account.lamports());
    invoke(
        &system_instruction::transfer(funding_account.key, target_account.key, lamports_diff),
        &[
            funding_account.clone(),
            target_account.clone(),
            system_program.clone(),
        ],
    )?;

    target_account.realloc(new_size, false)?;

    Ok(())
}

/// Verify the derivation of the seeds against the given account.
pub fn assert_derivation(
    program_id: &Pubkey,
    account: &Pubkey,
    path: &[&[u8]],
) -> Result<u8, ProgramError> {
    let (key, bump) = Pubkey::find_program_address(path, program_id);
    if key != *account {
        return Err(RuleSetError::DerivedKeyInvalid.into());
    }
    Ok(bump)
}

/// Assert that the given account is owned by the given pubkey.
pub fn assert_owned_by(account: &AccountInfo, owner: &Pubkey) -> ProgramResult {
    if account.owner != owner {
        Err(RuleSetError::IncorrectOwner.into())
    } else {
        Ok(())
    }
}

/// Compute the root of a Merkle tree given a leaf and a proof.  Uses a constant value
/// of 0x01 as an input to the hashing function along with the values to be hashed.
pub fn compute_merkle_root(leaf: &Pubkey, merkle_proof: &ProofInfo) -> [u8; 32] {
    let mut computed_hash = leaf.to_bytes();
    for proof_element in merkle_proof.proof.iter() {
        if computed_hash <= *proof_element {
            // Hash(current computed hash + current element of the proof).
            computed_hash =
                solana_program::keccak::hashv(&[&[0x01], &computed_hash, proof_element]).0;
        } else {
            // Hash(current element of the proof + current computed hash).
            computed_hash =
                solana_program::keccak::hashv(&[&[0x01], proof_element, &computed_hash]).0;
        }
    }

    computed_hash
}
