//! Utilities for the program
use crate::{
    error::RuleSetError,
    payload::ProofInfo,
    state::{
        Rule, RuleSetHeader, RuleSetRevisionMapV1, RuleSetV1, RULE_SET_REV_MAP_VERSION,
        RULE_SET_SERIALIZED_HEADER_LEN,
    },
};
use borsh::BorshDeserialize;
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
// TODO: Uncomment this when the syscall is available.
//use solana_zk_token_sdk::curve25519::curve_syscall_traits::CURVE25519_EDWARDS;

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

/// Get a revision map by looking at the header, finding its location, and deserializing it.
pub fn get_existing_revision_map(
    rule_set_pda_info: &AccountInfo,
) -> Result<(RuleSetRevisionMapV1, usize), ProgramError> {
    // Mutably borrow the existing `RuleSet` PDA data.
    let data = rule_set_pda_info
        .data
        .try_borrow()
        .map_err(|_| ProgramError::AccountBorrowFailed)?;

    // Deserialize header.
    let header = if data.len() >= RULE_SET_SERIALIZED_HEADER_LEN {
        RuleSetHeader::try_from_slice(&data[..RULE_SET_SERIALIZED_HEADER_LEN])?
    } else {
        return Err(RuleSetError::DataTypeMismatch.into());
    };

    // Get revision map version location from header and use it check revision map version.
    match data.get(header.rev_map_version_location) {
        Some(&RULE_SET_REV_MAP_VERSION) => {
            // Increment starting location by size of the revision map version.
            let start = header
                .rev_map_version_location
                .checked_add(1)
                .ok_or(RuleSetError::NumericalOverflow)?;

            // Deserialize revision map.
            if start < data.len() {
                let revision_map = RuleSetRevisionMapV1::try_from_slice(&data[start..])?;
                Ok((revision_map, header.rev_map_version_location))
            } else {
                Err(RuleSetError::DataTypeMismatch.into())
            }
        }
        Some(_) => Err(RuleSetError::UnsupportedRuleSetRevMapVersion.into()),
        None => Err(RuleSetError::DataTypeMismatch.into()),
    }
}

/// Get the latest revision number stored on the revision map.
///
/// This will first deserialize the header to find the map location and then deserialize the
/// revision map.
pub fn get_latest_revision(rule_set_pda_info: &AccountInfo) -> Result<Option<usize>, ProgramError> {
    let (revision_map, _) = get_existing_revision_map(rule_set_pda_info)?;

    match revision_map.rule_set_revisions.len() {
        // we should always have at least one revision
        0 => Err(RuleSetError::RuleSetRevisionNotAvailable.into()),
        // determine the index of the last revision
        length => Ok(Some(length - 1)),
    }
}

/// Return whether the pubkey is on the Edwards 25519 curve.
pub fn is_on_curve(pubkey: &Pubkey) -> bool {
    let _point = pubkey.to_bytes();
    let mut _validate_result = 0u8;
    // TODO: Uncomment this when the syscall is available.
    // let result = unsafe {
    //     solana_program::syscalls::sol_curve_validate_point(
    //         CURVE25519_EDWARDS,
    //         &point as *const u8,
    //         &mut validate_result,
    //     )
    // };

    // For now return false instead of checking the result.
    // result == 0
    false
}

/// See if a slice contains all zeroes.  Useful for checking an account's data.
pub fn is_zeroed(buf: &[u8]) -> bool {
    const ZEROS_LEN: usize = 1024;
    const ZEROS: [u8; ZEROS_LEN] = [0; ZEROS_LEN];

    let mut chunks = buf.chunks_exact(ZEROS_LEN);

    #[allow(clippy::indexing_slicing)]
    {
        chunks.all(|chunk| chunk == &ZEROS[..])
            && chunks.remainder() == &ZEROS[..chunks.remainder().len()]
    }
}

/// This function returns the rule for an operation by recursively searching through fallbacks
pub fn get_operation(operation: String, rule_set: &RuleSetV1) -> Result<&Rule, ProgramError> {
    let rule = rule_set.get(operation.to_string());

    match rule {
        Some(Rule::Namespace) => {
            // Check for a ':' namespace separator. If it exists try to operation namespace to see if
            // a fallback exists. E.g. 'transfer:owner' will check for a fallback for 'transfer'.
            // If it doesn't exist then fail.
            let split = operation.split(':').collect::<Vec<&str>>();
            if split.len() > 1 {
                get_operation(split[0].to_owned(), rule_set)
            } else {
                Err(RuleSetError::OperationNotFound.into())
            }
        }
        Some(r) => Ok(r),
        None => Err(RuleSetError::OperationNotFound.into()),
    }
}
