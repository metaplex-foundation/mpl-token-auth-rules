//! The processors for the Rule Set program instructions.   See state module for description of PDA memory layout.
use std::collections::HashMap;

use crate::{
    error::RuleSetError,
    instruction::{
        Context, CreateOrUpdate, CreateOrUpdateArgs, PuffRuleSet, PuffRuleSetArgs,
        RuleSetInstruction, Validate, ValidateArgs, WriteToBuffer, WriteToBufferArgs,
    },
    pda::{PREFIX, STATE_PDA},
    state::{
        RuleSetHeader, RuleSetRevisionMapV1, RuleSetV1, CHUNK_SIZE, RULE_SET_LIB_VERSION,
        RULE_SET_REV_MAP_VERSION, RULE_SET_SERIALIZED_HEADER_LEN,
    },
    utils::{
        assert_derivation, create_or_allocate_account_raw, get_existing_revision_map,
        get_operation, is_zeroed, resize_or_reallocate_account_raw,
    },
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
            RuleSetInstruction::PuffRuleSet(args) => {
                msg!("Instruction: PuffRuleSet");
                puff_rule_set(program_id, accounts, args)
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
    let context = CreateOrUpdate::to_context(accounts)?;

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

    // Get new or existing revision map.
    let revision_map = if ctx.accounts.rule_set_pda_info.data_is_empty()
        || is_zeroed(&ctx.accounts.rule_set_pda_info.data.borrow())
    {
        let mut revision_map = RuleSetRevisionMapV1::default();

        // Initially set the latest revision location to a the value right after the header.
        revision_map
            .rule_set_revisions
            .push(RULE_SET_SERIALIZED_HEADER_LEN);
        revision_map
    } else {
        // Get existing revision map and its serialized length.
        let (mut revision_map, existing_rev_map_loc) =
            get_existing_revision_map(ctx.accounts.rule_set_pda_info)?;

        // The next `RuleSet` revision will start where the existing revision map was.
        revision_map.rule_set_revisions.push(existing_rev_map_loc);
        revision_map
    };

    // Borsh serialize (or re-serialize) the revision map.
    let mut serialized_rev_map = Vec::new();
    revision_map
        .serialize(&mut serialized_rev_map)
        .map_err(|_| RuleSetError::BorshSerializationError)?;

    // Get new user-pre-serialized `RuleSet` data length based on whether it's in a buffer account
    // or provided as an argument.
    let new_rule_set_data_len = match ctx.accounts.buffer_pda_info {
        Some(account_info) => account_info.data_len(),
        None => serialized_rule_set.len(),
    };

    // Determine size needed for PDA: next revision location (which is:
    // (RULE_SET_SERIALIZED_HEADER_LEN || existing latest revision map location)) +
    // 2 bytes for version numbers + length of the serialized revision map +
    // length of user-pre-serialized `RuleSet`.
    let new_pda_data_len = revision_map
        .rule_set_revisions
        .last()
        .ok_or(RuleSetError::RuleSetRevisionNotAvailable)?
        .checked_add(2)
        .and_then(|len| len.checked_add(serialized_rev_map.len()))
        .and_then(|len| len.checked_add(new_rule_set_data_len))
        .ok_or(RuleSetError::NumericalOverflow)?;

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

    // Write all the data to the PDA.  The user-pre-serialized `RuleSet` is either in a buffer
    // account or provided as an argument.
    match ctx.accounts.buffer_pda_info {
        Some(account_info) => {
            write_data_to_pda(
                ctx.accounts.rule_set_pda_info,
                *revision_map
                    .rule_set_revisions
                    .last()
                    .ok_or(RuleSetError::RuleSetRevisionNotAvailable)?,
                &serialized_rev_map,
                &account_info.data.borrow(),
            )?;
        }
        None => {
            write_data_to_pda(
                ctx.accounts.rule_set_pda_info,
                *revision_map
                    .rule_set_revisions
                    .last()
                    .ok_or(RuleSetError::RuleSetRevisionNotAvailable)?,
                &serialized_rev_map,
                &serialized_rule_set,
            )?;
        }
    };

    Ok(())
}

// Function to match on `ValidateArgs` version and call correct implementation.
fn validate<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    args: ValidateArgs,
) -> ProgramResult {
    let context = Validate::to_context(accounts)?;

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
        rule_set_revision,
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

    // Get existing revision map and its serialized length.
    let (revision_map, rev_map_location) =
        get_existing_revision_map(ctx.accounts.rule_set_pda_info)?;

    // Use the user-provided revision number to look up the `RuleSet` revision location in the PDA.
    let (start, end) = match rule_set_revision {
        Some(revision) => {
            let start = revision_map
                .rule_set_revisions
                .get(revision)
                .ok_or(RuleSetError::RuleSetRevisionNotAvailable)?;

            let end_index = revision
                .checked_add(1)
                .ok_or(RuleSetError::NumericalOverflow)?;

            let end = revision_map
                .rule_set_revisions
                .get(end_index)
                .unwrap_or(&rev_map_location);
            (*start, *end)
        }
        None => {
            let start = revision_map
                .rule_set_revisions
                .last()
                .ok_or(RuleSetError::RuleSetRevisionNotAvailable)?;
            (*start, rev_map_location)
        }
    };

    // Mutably borrow the existing `RuleSet` PDA data.
    let data = ctx
        .accounts
        .rule_set_pda_info
        .data
        .try_borrow()
        .map_err(|_| ProgramError::AccountBorrowFailed)?;

    // Check `RuleSet` lib version.
    let rule_set = match data.get(start) {
        Some(&RULE_SET_LIB_VERSION) => {
            // Increment starting location by size of lib version.
            let start = start
                .checked_add(1)
                .ok_or(RuleSetError::NumericalOverflow)?;

            // Deserialize `RuleSet`.
            if end < ctx.accounts.rule_set_pda_info.data_len() {
                rmp_serde::from_slice::<RuleSetV1>(&data[start..end])
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
    let rule = get_operation(operation, &rule_set)?;

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
    let context = WriteToBuffer::to_context(accounts)?;

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
    let offset = if overwrite {
        0
    } else {
        ctx.accounts.buffer_pda_info.data_len()
    };

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

// Function to match on `PuffRuleSet` version and call correct implementation.
fn puff_rule_set<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    args: PuffRuleSetArgs,
) -> ProgramResult {
    let context = PuffRuleSet::to_context(accounts)?;

    match args {
        PuffRuleSetArgs::V1 { .. } => puff_rule_set_v1(program_id, context, args),
    }
}

/// V1 implementation of the `puff_rule_set` instruction.
fn puff_rule_set_v1(
    program_id: &Pubkey,
    ctx: Context<PuffRuleSet>,
    args: PuffRuleSetArgs,
) -> ProgramResult {
    let PuffRuleSetArgs::V1 { rule_set_name } = args;

    if !ctx.accounts.payer_info.is_signer {
        return Err(RuleSetError::PayerIsNotSigner.into());
    }

    // Check `RuleSet` account info derivation.
    let bump = assert_derivation(
        program_id,
        ctx.accounts.rule_set_pda_info.key,
        &[
            PREFIX.as_bytes(),
            ctx.accounts.payer_info.key.as_ref(),
            rule_set_name.as_bytes(),
        ],
    )?;

    let rule_set_seeds = &[
        PREFIX.as_ref(),
        ctx.accounts.payer_info.key.as_ref(),
        rule_set_name.as_ref(),
        &[bump],
    ];

    // Create or allocate, resize or reallocate the `RuleSet` PDA.
    if ctx.accounts.rule_set_pda_info.data_is_empty() {
        create_or_allocate_account_raw(
            *program_id,
            ctx.accounts.rule_set_pda_info,
            ctx.accounts.system_program_info,
            ctx.accounts.payer_info,
            CHUNK_SIZE,
            rule_set_seeds,
        )
    } else {
        resize_or_reallocate_account_raw(
            ctx.accounts.rule_set_pda_info,
            ctx.accounts.payer_info,
            ctx.accounts.system_program_info,
            ctx.accounts
                .rule_set_pda_info
                .data_len()
                .checked_add(CHUNK_SIZE)
                .ok_or(RuleSetError::NumericalOverflow)?,
        )
    }
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

// Write the `RuleSet` lib version, a serialized `RuleSet`, the revision map version,
// a revision map, and a header to the `RuleSet` PDA.
fn write_data_to_pda(
    rule_set_pda_info: &AccountInfo,
    starting_location: usize,
    serialized_rev_map: &[u8],
    serialized_rule_set: &[u8],
) -> ProgramResult {
    // Mutably borrow the `RuleSet` PDA data.
    let data = &mut rule_set_pda_info
        .try_borrow_mut_data()
        .map_err(|_| ProgramError::AccountBorrowFailed)?;

    // Copy `RuleSet` lib version to PDA account starting at the location stored in the revision
    // map for the latest revision.
    let start = starting_location;
    let end = start
        .checked_add(1)
        .ok_or(RuleSetError::NumericalOverflow)?;
    if end <= data.len() {
        sol_memcpy(&mut data[start..end], &[RULE_SET_LIB_VERSION], 1);
    } else {
        return Err(RuleSetError::DataSliceUnexpectedIndexError.into());
    }

    // Copy serialized `RuleSet` to PDA account.
    let start = end;
    let end = start
        .checked_add(serialized_rule_set.len())
        .ok_or(RuleSetError::NumericalOverflow)?;
    if end <= data.len() {
        sol_memcpy(
            &mut data[start..end],
            serialized_rule_set,
            serialized_rule_set.len(),
        );
    } else {
        return Err(RuleSetError::DataSliceUnexpectedIndexError.into());
    }

    // Copy the revision map version to PDA account.
    let start = end;
    let end = start
        .checked_add(1)
        .ok_or(RuleSetError::NumericalOverflow)?;
    if end <= data.len() {
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
    if end <= data.len() {
        sol_memcpy(
            &mut data[start..end],
            serialized_rev_map,
            serialized_rev_map.len(),
        );
    } else {
        return Err(RuleSetError::DataSliceUnexpectedIndexError.into());
    }

    let start = 0;
    let end = RULE_SET_SERIALIZED_HEADER_LEN;
    if end <= data.len() {
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
