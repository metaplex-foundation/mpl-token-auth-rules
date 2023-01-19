//! The processors for the Rule Set program instructions.
use std::collections::HashMap;

use crate::{
    error::RuleSetError,
    instruction::{
        Context, CreateOrUpdate, CreateOrUpdateArgs, RuleSetInstruction, Validate, ValidateArgs,
        WriteToBuffer, WriteToBufferArgs,
    },
    pda::{PREFIX, STATE_PDA},
    state::{
        RuleSetHeader, RuleSetRevisionMapV1, RuleSetV1, RULE_SET_LIB_VERSION,
        RULE_SET_REV_MAP_VERSION, RULE_SET_SERIALIZED_HEADER_LEN,
    },
    utils::{assert_derivation, create_or_allocate_account_raw, resize_or_reallocate_account_raw},
    MAX_NAME_LENGTH,
};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    program_memory::{sol_memcmp, sol_memcpy},
    pubkey::{Pubkey, PUBKEY_BYTES},
};

/// The generic processor struct.
pub struct Processor;
impl Processor {
    /// The main entrypoint for the Rule Set program that matches on the instruction type and args
    pub fn process_instruction<'a>(
        program_id: &Pubkey,
        accounts: &'a [AccountInfo<'a>],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction = RuleSetInstruction::try_from_slice(instruction_data)?;
        match instruction {
            RuleSetInstruction::CreateOrUpdate(args) => {
                msg!("Instruction: CreateOrUpdate");
                create_or_update(program_id, accounts, args)
            }
            RuleSetInstruction::Validate(args) => {
                msg!("Instruction: Validate");
                validate(program_id, accounts, args)
            }
            RuleSetInstruction::WriteToBuffer(args) => {
                msg!("Instruction: WriteToBuffer");
                write_to_buffer(program_id, accounts, args)
            }
        }
    }
}

// Function to match on `CreateOrUpdateArgs` version and call correct implementation.
fn create_or_update<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    args: CreateOrUpdateArgs,
) -> ProgramResult {
    let context = CreateOrUpdate::as_context(accounts)?;

    match args {
        CreateOrUpdateArgs::V1 { .. } => create_or_update_v1(program_id, context, args),
    }
}

/// V1 implementation of the `create` instruction.
fn create_or_update_v1(
    program_id: &Pubkey,
    ctx: Context<CreateOrUpdate>,
    args: CreateOrUpdateArgs,
) -> ProgramResult {
    // Get the V1 arguments for the instruction.
    let CreateOrUpdateArgs::V1 {
        serialized_rule_set,
    } = args;

    if !ctx.accounts.payer_info.is_signer {
        return Err(RuleSetError::PayerIsNotSigner.into());
    }

    // Deserialize `RuleSet`.
    let rule_set = match ctx.accounts.buffer_pda_info {
        Some(account_info) => rmp_serde::from_slice::<RuleSetV1>(&account_info.data.borrow())
            .map_err(|_| RuleSetError::MessagePackDeserializationError)?,
        None => rmp_serde::from_slice(&serialized_rule_set)
            .map_err(|_| RuleSetError::MessagePackDeserializationError)?,
    };

    if rule_set.name().len() > MAX_NAME_LENGTH {
        return Err(RuleSetError::NameTooLong.into());
    }

    // Make sure we know how to work with this RuleSet.
    if rule_set.lib_version() != RULE_SET_LIB_VERSION {
        return Err(RuleSetError::UnsupportedRuleSetVersion.into());
    }

    // The payer/signer must be the `RuleSet` owner.
    if ctx.accounts.payer_info.key != rule_set.owner() {
        return Err(RuleSetError::RuleSetOwnerMismatch.into());
    }

    // Check `RuleSet` account info derivation.
    let bump = assert_derivation(
        program_id,
        ctx.accounts.rule_set_pda_info.key,
        &[
            PREFIX.as_bytes(),
            ctx.accounts.payer_info.key.as_ref(),
            rule_set.name().as_bytes(),
        ],
    )?;

    let rule_set_seeds = &[
        PREFIX.as_ref(),
        ctx.accounts.payer_info.key.as_ref(),
        rule_set.name().as_ref(),
        &[bump],
    ];

    let new_rule_set_data_len = match ctx.accounts.buffer_pda_info {
        Some(account_info) => account_info.data_len(),
        None => serialized_rule_set.len(),
    };

    let existing_pda_data_len = ctx.accounts.rule_set_pda_info.data_len();

    let (revision_map, serialized_rev_map, new_pda_data_len) =
        if ctx.accounts.rule_set_pda_info.data_is_empty() {
            let mut revision_map = RuleSetRevisionMapV1::default();

            // Initially set the first revision location to a the value right after the header.
            revision_map
                .rule_set_revisions
                .push(RULE_SET_SERIALIZED_HEADER_LEN);
            revision_map.max_revision = 0;

            // Borsh serialize the revision map.
            let mut serialized_rev_map = Vec::new();
            revision_map
                .serialize(&mut serialized_rev_map)
                .map_err(|_| RuleSetError::BorshSerializationError)?;

            // Determine size needed for PDA: length of serialized header +
            // 2 bytes for version numbers + length of the serialized revision map +
            // length of user-pre-serialized `RuleSet`.
            let new_pda_data_len = RULE_SET_SERIALIZED_HEADER_LEN
                .checked_add(2)
                .and_then(|len| len.checked_add(serialized_rev_map.len()))
                .and_then(|len| len.checked_add(new_rule_set_data_len))
                .ok_or(RuleSetError::NumericalOverflow)?;

            (revision_map, serialized_rev_map, new_pda_data_len)
        } else {
            // Get existing revision map and its serialized length.
            let (mut revision_map, existing_serialized_rev_map_len) =
                get_existing_revision_map(ctx.accounts.rule_set_pda_info)?;

            // Update the revision map: Increment max revision and save the new `RuleSet` revision's
            // location which is the end of the current PDA data length.
            revision_map.max_revision = revision_map
                .max_revision
                .checked_add(1)
                .ok_or(RuleSetError::NumericalOverflow)?;
            revision_map.rule_set_revisions.push(existing_pda_data_len);

            // Borsh re-serialize the revision map.
            let mut serialized_rev_map = Vec::new();
            revision_map
                .serialize(&mut serialized_rev_map)
                .map_err(|_| RuleSetError::BorshSerializationError)?;

            // Determine size needed for PDA: existing data length -
            // length of the old serialized revision map + length of the new serialized revision map +
            // 1 byte for version number + length of user-pre-serialized `RuleSet`.
            let new_pda_data_len = existing_pda_data_len
                .checked_sub(existing_serialized_rev_map_len)
                .and_then(|len| len.checked_add(serialized_rev_map.len()))
                .and_then(|len| len.checked_add(1))
                .and_then(|len| len.checked_add(new_rule_set_data_len))
                .ok_or(RuleSetError::NumericalOverflow)?;

            (revision_map, serialized_rev_map, new_pda_data_len)
        };

    // Create or allocate, resize or reallocate the `RuleSet` PDA.
    if ctx.accounts.rule_set_pda_info.data_is_empty() {
        create_or_allocate_account_raw(
            *program_id,
            ctx.accounts.rule_set_pda_info,
            ctx.accounts.system_program_info,
            ctx.accounts.payer_info,
            new_pda_data_len,
            rule_set_seeds,
        )?;
    } else {
        resize_or_reallocate_account_raw(
            ctx.accounts.rule_set_pda_info,
            ctx.accounts.payer_info,
            ctx.accounts.system_program_info,
            new_pda_data_len,
        )?;
    }

    // Mutably borrow the `RuleSet` PDA data.
    let data = &mut ctx
        .accounts
        .rule_set_pda_info
        .try_borrow_mut_data()
        .map_err(|_| ProgramError::AccountBorrowFailed)?;

    // Copy `RuleSet` lib version to PDA account starting at the location stored in the revision
    // map for the max revision.
    let start = revision_map.rule_set_revisions[revision_map.max_revision];
    let end = start
        .checked_add(1)
        .ok_or(RuleSetError::NumericalOverflow)?;
    if end <= new_pda_data_len {
        sol_memcpy(&mut data[start..end], &[RULE_SET_LIB_VERSION], 1);
    } else {
        return Err(RuleSetError::DataSliceUnexpectedIndexError.into());
    }

    // Copy user-pre-serialized `RuleSet` to PDA account.
    let start = end;
    let end = start
        .checked_add(new_rule_set_data_len)
        .ok_or(RuleSetError::NumericalOverflow)?;
    if end <= new_pda_data_len {
        match ctx.accounts.buffer_pda_info {
            Some(account_info) => {
                sol_memcpy(
                    &mut data[start..end],
                    &account_info.data.borrow(),
                    new_rule_set_data_len,
                );
            }
            None => {
                sol_memcpy(
                    &mut data[start..end],
                    &serialized_rule_set,
                    new_rule_set_data_len,
                );
            }
        }
    } else {
        return Err(RuleSetError::DataSliceUnexpectedIndexError.into());
    }

    // Copy the revision map version to PDA account.
    let start = end;
    let end = start
        .checked_add(1)
        .ok_or(RuleSetError::NumericalOverflow)?;
    if end <= new_pda_data_len {
        sol_memcpy(&mut data[start..end], &[RULE_SET_REV_MAP_VERSION], 1);
    } else {
        return Err(RuleSetError::DataSliceUnexpectedIndexError.into());
    }

    // Create a new header holding the location of the revision map version.
    let header = RuleSetHeader::new(start);

    // Borsh serialize the header.
    let mut serialized_header = Vec::new();
    header
        .serialize(&mut serialized_header)
        .map_err(|_| RuleSetError::BorshSerializationError)?;

    // Copy the serialized revision map to PDA account.
    let start = end;
    let end = start
        .checked_add(serialized_rev_map.len())
        .ok_or(RuleSetError::NumericalOverflow)?;
    if end <= new_pda_data_len {
        sol_memcpy(
            &mut data[start..end],
            &serialized_rev_map,
            serialized_rev_map.len(),
        );
    } else {
        return Err(RuleSetError::DataSliceUnexpectedIndexError.into());
    }

    let start = 0;
    let end = RULE_SET_SERIALIZED_HEADER_LEN;
    if end <= new_pda_data_len {
        sol_memcpy(
            &mut data[start..end],
            &serialized_header,
            serialized_header.len(),
        );
    } else {
        return Err(RuleSetError::DataSliceUnexpectedIndexError.into());
    }

    Ok(())
}

// Function to match on `ValidateArgs` version and call correct implementation.
fn validate<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    args: ValidateArgs,
) -> ProgramResult {
    let context = Validate::as_context(accounts)?;

    match args {
        ValidateArgs::V1 { .. } => validate_v1(program_id, context, args),
    }
}

/// V1 implementation of the `validate` instruction.
fn validate_v1(program_id: &Pubkey, ctx: Context<Validate>, args: ValidateArgs) -> ProgramResult {
    // Get the V1 arguments for the instruction.
    let ValidateArgs::V1 {
        operation,
        payload,
        update_rule_state,
        rule_set_version,
    } = args;

    // If state is being updated for any `Rule`s, the payer must be present and must be a signer so
    // that the `RuleSet` state PDA can be created or reallocated.
    if update_rule_state {
        if let Some(payer_info) = ctx.accounts.payer_info {
            if !payer_info.is_signer {
                return Err(RuleSetError::PayerIsNotSigner.into());
            }
        } else {
            return Err(ProgramError::NotEnoughAccountKeys);
        }
    }

    // `RuleSet` must be owned by this program.
    if *ctx.accounts.rule_set_pda_info.owner != crate::ID {
        return Err(RuleSetError::IncorrectOwner.into());
    }

    // `RuleSet` must not be empty.
    if ctx.accounts.rule_set_pda_info.data_is_empty() {
        return Err(RuleSetError::DataIsEmpty.into());
    }

    let data_len = ctx.accounts.rule_set_pda_info.data_len();

    // Get existing revision map and its serialized length.
    let (revision_map, _) = get_existing_revision_map(ctx.accounts.rule_set_pda_info)?;

    // Use the user-provided revision number to look up the `RuleSet` revision location in the PDA.
    let starting_loc = match rule_set_version {
        Some(version) => revision_map
            .rule_set_revisions
            .get(version)
            .ok_or::<ProgramError>(RuleSetError::RuleSetRevNotAvailable.into())?,
        None => revision_map
            .rule_set_revisions
            .last()
            .ok_or::<ProgramError>(RuleSetError::RuleSetRevNotAvailable.into())?,
    };

    // Mutably borrow the existing `RuleSet` PDA data.
    let data = ctx
        .accounts
        .rule_set_pda_info
        .data
        .try_borrow()
        .map_err(|_| ProgramError::AccountBorrowFailed)?;

    // Check `RuleSet` lib version.
    let rule_set = match data.get(*starting_loc) {
        Some(&RULE_SET_LIB_VERSION) => {
            // Increment starting location by size of lib version.
            let starting_loc = starting_loc
                .checked_add(1)
                .ok_or(RuleSetError::NumericalOverflow)?;

            // Deserialize `RuleSet`.
            if data_len > starting_loc {
                rmp_serde::from_slice::<RuleSetV1>(&data[starting_loc..])
                    .map_err(|_| RuleSetError::MessagePackDeserializationError)?
            } else {
                return Err(RuleSetError::DataTypeMismatch.into());
            }
        }
        Some(_) => return Err(RuleSetError::UnsupportedRuleSetVersion.into()),
        None => return Err(RuleSetError::DataTypeMismatch.into()),
    };

    // Check `RuleSet` account info derivation.
    let _bump = assert_derivation(
        program_id,
        ctx.accounts.rule_set_pda_info.key,
        &[
            PREFIX.as_bytes(),
            rule_set.owner().as_ref(),
            rule_set.name().as_bytes(),
        ],
    )?;

    // If `RuleSet` state is to be updated, check account info derivation.
    if update_rule_state {
        if let Some(rule_set_state_pda_info) = ctx.accounts.rule_set_state_pda_info {
            let _bump = assert_derivation(
                program_id,
                rule_set_state_pda_info.key,
                &[
                    STATE_PDA.as_bytes(),
                    rule_set.owner().as_ref(),
                    rule_set.name().as_bytes(),
                    ctx.accounts.mint_info.key.as_ref(),
                ],
            )?;
        } else {
            return Err(ProgramError::NotEnoughAccountKeys);
        }
    }

    // Convert remaining `Rule` accounts into a map of `Pubkey`s to the corresponding
    // `AccountInfo`s.  This makes it easy to pass the account infos into validation functions
    // since they store the `Pubkey`s.
    let accounts_map = ctx
        .remaining_accounts
        .iter()
        .map(|account| (*account.key, *account))
        .collect::<HashMap<Pubkey, &AccountInfo>>();

    // Get the `Rule` from the `RuleSet` based on the user-specified operation.
    let rule = rule_set
        .get(operation)
        .ok_or(RuleSetError::OperationNotFound)?;

    // Validate the `Rule`.
    if let Err(err) = rule.validate(
        &accounts_map,
        &payload,
        update_rule_state,
        &ctx.accounts.rule_set_state_pda_info,
        &ctx.accounts.rule_authority_info,
    ) {
        msg!("Failed to validate: {}", err);
        return Err(err);
    }

    Ok(())
}

// Function to match on `WriteToBuffer` version and call correct implementation.
fn write_to_buffer<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    args: WriteToBufferArgs,
) -> ProgramResult {
    let context = WriteToBuffer::as_context(accounts)?;

    match args {
        WriteToBufferArgs::V1 { .. } => write_to_buffer_v1(program_id, context, args),
    }
}

/// V1 implementation of the `write_to_buffer` instruction.
fn write_to_buffer_v1(
    program_id: &Pubkey,
    ctx: Context<WriteToBuffer>,
    args: WriteToBufferArgs,
) -> ProgramResult {
    let WriteToBufferArgs::V1 {
        serialized_rule_set,
        overwrite,
    } = args;

    if !ctx.accounts.payer_info.is_signer {
        return Err(RuleSetError::PayerIsNotSigner.into());
    }

    // Check buffer account info derivation.
    let bump = assert_derivation(
        program_id,
        ctx.accounts.buffer_pda_info.key,
        &[PREFIX.as_bytes(), ctx.accounts.payer_info.key.as_ref()],
    )?;

    let buffer_seeds = &[
        PREFIX.as_ref(),
        ctx.accounts.payer_info.key.as_ref(),
        &[bump],
    ];

    // Fetch the offset before we realloc so we get the accurate account length.
    let offset = ctx.accounts.buffer_pda_info.data_len();

    // Create or allocate, resize or reallocate buffer PDA.
    if ctx.accounts.buffer_pda_info.data_is_empty() {
        create_or_allocate_account_raw(
            *program_id,
            ctx.accounts.buffer_pda_info,
            ctx.accounts.system_program_info,
            ctx.accounts.payer_info,
            serialized_rule_set.len(),
            buffer_seeds,
        )?;
    } else if overwrite {
        resize_or_reallocate_account_raw(
            ctx.accounts.buffer_pda_info,
            ctx.accounts.payer_info,
            ctx.accounts.system_program_info,
            serialized_rule_set.len(),
        )?;
    } else {
        resize_or_reallocate_account_raw(
            ctx.accounts.buffer_pda_info,
            ctx.accounts.payer_info,
            ctx.accounts.system_program_info,
            ctx.accounts
                .buffer_pda_info
                .data_len()
                .checked_add(serialized_rule_set.len())
                .ok_or(RuleSetError::NumericalOverflow)?,
        )?;
    }

    msg!(
        "Writing {:?} bytes at offset {:?}",
        serialized_rule_set.len(),
        offset
    );
    // Copy user-pre-serialized RuleSet to PDA account.
    sol_memcpy(
        &mut ctx.accounts.buffer_pda_info.try_borrow_mut_data().unwrap()[offset..],
        &serialized_rule_set,
        serialized_rule_set.len(),
    );

    Ok(())
}

/// Convenience function for accessing the next item in an [`AccountInfo`]
/// iterator and validating whether the account is present or not.
///
/// This relies on the client setting the `crate::id()` as the pubkey for
/// accounts that are not set, which effectively allows us to use positional
/// optional accounts.
pub fn next_optional_account_info<'a, 'b, I: Iterator<Item = &'a AccountInfo<'b>>>(
    iter: &mut I,
) -> Result<Option<I::Item>, ProgramError> {
    let account_info = iter.next().ok_or(ProgramError::NotEnoughAccountKeys)?;

    Ok(if cmp_pubkeys(account_info.key, &crate::id()) {
        None
    } else {
        Some(account_info)
    })
}

/// Convenience function for comparing two [`Pubkey`]s.
pub fn cmp_pubkeys(a: &Pubkey, b: &Pubkey) -> bool {
    sol_memcmp(a.as_ref(), b.as_ref(), PUBKEY_BYTES) == 0
}

fn get_existing_revision_map(
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
                // Safe subtraction because we just checked for `<`.
                let serialized_rev_map_len = data.len() - start;
                Ok((revision_map, serialized_rev_map_len))
            } else {
                Err(RuleSetError::DataTypeMismatch.into())
            }
        }
        Some(_) => return Err(RuleSetError::UnsupportedRuleSetVersion.into()),
        None => return Err(RuleSetError::DataTypeMismatch.into()),
    }
}
