use borsh::BorshSerialize;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    program_memory::sol_memcpy, pubkey::Pubkey,
};

use crate::{
    error::RuleSetError,
    instruction::{Context, CreateOrUpdate, CreateOrUpdateArgs},
    pda::PREFIX,
    state::{
        RuleSetHeader, RuleSetRevisionMapV1, RuleSetV1, RULE_SET_REV_MAP_VERSION,
        RULE_SET_SERIALIZED_HEADER_LEN,
    },
    state_v2::{RuleSetV2, SIZE_U64},
    utils::{
        assert_derivation, create_or_allocate_account_raw, get_existing_revision_map, is_zeroed,
        resize_or_reallocate_account_raw,
    },
    LibVersion, MAX_NAME_LENGTH,
};

const ALIGNMENT_PADDING: usize = (SIZE_U64 * 2) - RULE_SET_SERIALIZED_HEADER_LEN;

// Function to match on `CreateOrUpdateArgs` version and call correct implementation.
pub(crate) fn create_or_update<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    args: CreateOrUpdateArgs,
) -> ProgramResult {
    let context = CreateOrUpdate::to_context(accounts)?;

    match args {
        CreateOrUpdateArgs::V1 {
            serialized_rule_set,
        } => create_or_update_v1(program_id, context, serialized_rule_set),
    }
}

/// V1 implementation of the `create` instruction.
fn create_or_update_v1(
    program_id: &Pubkey,
    ctx: Context<CreateOrUpdate>,
    serialized_rule_set: Vec<u8>,
) -> ProgramResult {
    if !ctx.accounts.payer_info.is_signer {
        return Err(RuleSetError::PayerIsNotSigner.into());
    }

    // Deserialize the `RuleSet`.
    let (rule_set_version, rule_set_name, owner) = match ctx.accounts.buffer_pda_info {
        Some(account_info) => {
            if let Ok(rule_set) = rmp_serde::from_slice::<RuleSetV1>(&(*account_info.data).borrow())
            {
                (
                    LibVersion::try_from(rule_set.lib_version())?,
                    rule_set.name().to_string(),
                    *rule_set.owner(),
                )
            } else if let Ok(rule_set) = RuleSetV2::from_bytes(&(*account_info.data).borrow()) {
                (
                    LibVersion::try_from(rule_set.lib_version())?,
                    rule_set.name(),
                    *rule_set.owner,
                )
            } else {
                return Err(RuleSetError::MessagePackDeserializationError.into());
            }
        }
        None => {
            if let Ok(rule_set) = rmp_serde::from_slice::<RuleSetV1>(&serialized_rule_set) {
                (
                    LibVersion::try_from(rule_set.lib_version())?,
                    rule_set.name().to_string(),
                    *rule_set.owner(),
                )
            } else if let Ok(rule_set) = RuleSetV2::from_bytes(&serialized_rule_set) {
                (
                    LibVersion::try_from(rule_set.lib_version())?,
                    rule_set.name(),
                    *rule_set.owner,
                )
            } else {
                return Err(RuleSetError::MessagePackDeserializationError.into());
            }
        }
    };

    // Check that the name is not too long.
    if rule_set_name.len() > MAX_NAME_LENGTH {
        return Err(RuleSetError::NameTooLong.into());
    }

    // The payer/signer must be the `RuleSet` owner.
    if *ctx.accounts.payer_info.key != owner {
        return Err(RuleSetError::RuleSetOwnerMismatch.into());
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

    // Get new or existing revision map.
    let revision_map = if ctx.accounts.rule_set_pda_info.data_is_empty()
        || is_zeroed(&ctx.accounts.rule_set_pda_info.data.borrow())
    {
        let mut revision_map = RuleSetRevisionMapV1::default();

        // Initially set the latest revision location to a the value right after the header.
        revision_map.rule_set_revisions.push(
            RULE_SET_SERIALIZED_HEADER_LEN
                + if matches!(rule_set_version, LibVersion::V2) {
                    ALIGNMENT_PADDING
                } else {
                    0
                },
        );
        revision_map
    } else {
        // Get existing revision map and its serialized length.
        let (mut revision_map, existing_rev_map_loc) =
            get_existing_revision_map(ctx.accounts.rule_set_pda_info)?;

        // The next `RuleSet` revision will start where the existing revision map was + any
        // alignment required (V2 only)
        let alignment = if matches!(rule_set_version, LibVersion::V2) {
            let delta = existing_rev_map_loc
                .checked_rem(SIZE_U64)
                .ok_or(RuleSetError::NumericalOverflow)?;

            SIZE_U64
                .checked_sub(delta)
                .ok_or(RuleSetError::NumericalOverflow)?
        } else {
            0
        };

        revision_map
            .rule_set_revisions
            .push(existing_rev_map_loc + alignment);

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
    // (RULE_SET_SERIALIZED_HEADER_LEN || existing latest revision map location))
    //   + rule set lib version (optional)
    //   + revision map version
    //   + length of user-pre-serialized `RuleSet`
    //   + length of the serialized revision map
    let new_pda_data_len = revision_map
        .rule_set_revisions
        .last()
        .ok_or(RuleSetError::RuleSetRevisionNotAvailable)?
        .checked_add(if matches!(rule_set_version, LibVersion::V2) {
            // `RuleSetV2` already incorporates the lib_version as the
            // first byte of the serialized data, so we only add a byte
            // for the revision map version
            1
        } else {
            // `RuleSetV1` lib version + revision map version
            2
        })
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
                matches!(rule_set_version, LibVersion::V1),
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
                matches!(rule_set_version, LibVersion::V1),
            )?;
        }
    };

    Ok(())
}

// Write the `RuleSet` lib version, a serialized `RuleSet`, the revision map version,
// a revision map, and a header to the `RuleSet` PDA.
fn write_data_to_pda(
    rule_set_pda_info: &AccountInfo,
    starting_location: usize,
    serialized_rev_map: &[u8],
    serialized_rule_set: &[u8],
    write_lib_version: bool,
) -> ProgramResult {
    // Mutably borrow the `RuleSet` PDA data.
    let data = &mut rule_set_pda_info
        .try_borrow_mut_data()
        .map_err(|_| ProgramError::AccountBorrowFailed)?;

    // Copy `RuleSet` lib version to PDA account starting at the location stored in the revision
    // map for the latest revision.
    let start = if write_lib_version {
        let start = starting_location;
        let end = start
            .checked_add(1)
            .ok_or(RuleSetError::NumericalOverflow)?;
        if end <= data.len() {
            sol_memcpy(&mut data[start..end], &[LibVersion::V1 as u8], 1);
        } else {
            return Err(RuleSetError::DataSliceUnexpectedIndexError.into());
        }
        end
    } else {
        starting_location
    };

    // Copy serialized `RuleSet` to PDA account.
    //let start = end;
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
