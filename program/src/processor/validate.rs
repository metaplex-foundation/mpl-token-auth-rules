use std::collections::HashMap;

use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::{
    error::RuleSetError,
    instruction::{Context, Validate, ValidateArgs},
    payload::Payload,
    pda::{PREFIX, STATE_PDA},
    state::RuleSetV1,
    state_v2::RuleSetV2,
    types::{Assertable, LibVersion},
    utils::{assert_derivation, get_existing_revision_map},
};

// Function to match on `ValidateArgs` version and call correct implementation.
pub(crate) fn validate<'a>(
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
    let lib_version = match data.get(start) {
        Some(lib_version) => LibVersion::try_from(*lib_version)?,
        None => return Err(RuleSetError::DataTypeMismatch.into()),
    };

    match lib_version {
        LibVersion::V1 => {
            // Increment starting location by size of lib version.
            let start = start
                .checked_add(1)
                .ok_or(RuleSetError::NumericalOverflow)?;

            // Deserialize `RuleSet`.
            if end < ctx.accounts.rule_set_pda_info.data_len() {
                let rule_set = rmp_serde::from_slice::<RuleSetV1>(&data[start..end])
                    .map_err(|_| RuleSetError::MessagePackDeserializationError)?;
                // Validate the `Rule` and update the `RuleSet` state if requested.
                validate_rule(
                    program_id,
                    &ctx,
                    rule_set.name().to_string(),
                    *rule_set.owner(),
                    rule_set.get_operation(operation)? as &dyn Assertable,
                    payload,
                    update_rule_state,
                )
            } else {
                Err(RuleSetError::DataTypeMismatch.into())
            }
        }
        LibVersion::V2 => {
            if end < ctx.accounts.rule_set_pda_info.data_len() {
                let rule_set = RuleSetV2::from_bytes(&data[start..end])?;
                // Validate the `Rule` and update the `RuleSet` state if requested.
                validate_rule(
                    program_id,
                    &ctx,
                    rule_set.name(),
                    *rule_set.owner,
                    rule_set.get_operation(operation)? as &dyn Assertable,
                    payload,
                    update_rule_state,
                )
            } else {
                Err(RuleSetError::DataTypeMismatch.into())
            }
        }
    }
}

fn validate_rule(
    program_id: &Pubkey,
    ctx: &Context<Validate>,
    rule_set_name: String,
    owner: Pubkey,
    rule: &dyn Assertable,
    payload: Payload,
    update_rule_state: bool,
) -> ProgramResult {
    // Check `RuleSet` account info derivation.
    let _bump = assert_derivation(
        program_id,
        ctx.accounts.rule_set_pda_info.key,
        &[PREFIX.as_bytes(), owner.as_ref(), rule_set_name.as_bytes()],
    )?;

    // If `RuleSet` state is to be updated, check account info derivation.
    if update_rule_state {
        if let Some(rule_set_state_pda_info) = ctx.accounts.rule_set_state_pda_info {
            let _bump = assert_derivation(
                program_id,
                rule_set_state_pda_info.key,
                &[
                    STATE_PDA.as_bytes(),
                    owner.as_ref(),
                    rule_set_name.as_bytes(),
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
