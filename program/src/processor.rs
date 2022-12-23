use std::collections::HashMap;

use crate::{
    error::RuleSetError,
    instruction::{
        Context, CreateOrUpdate, CreateOrUpdateArgs, RuleSetInstruction, Validate, ValidateArgs,
    },
    pda::{PREFIX, STATE_PDA},
    state::{RuleSet, RULE_SET_VERSION},
    utils::{assert_derivation, create_or_allocate_account_raw, resize_or_reallocate_account_raw},
    MAX_NAME_LENGTH,
};
use borsh::BorshDeserialize;
use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    program_memory::{sol_memcmp, sol_memcpy},
    pubkey::{Pubkey, PUBKEY_BYTES},
};

pub struct Processor;
impl Processor {
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

    // Deserialize RuleSet.
    let rule_set: RuleSet = rmp_serde::from_slice(&serialized_rule_set)
        .map_err(|_| RuleSetError::MessagePackDeserializationError)?;

    if rule_set.name().len() > MAX_NAME_LENGTH {
        return Err(RuleSetError::NameTooLong.into());
    }

    // Make sure we know how to work with this RuleSet.
    if rule_set.version() != RULE_SET_VERSION {
        return Err(RuleSetError::UnsupportedRuleSetVersion.into());
    }

    // The payer/signer must be the RuleSet owner.
    if ctx.accounts.payer_info.key != rule_set.owner() {
        return Err(RuleSetError::RuleSetOwnerMismatch.into());
    }

    // Check RuleSet account info derivation.
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

    // Create or allocate, resize or reallocate RuleSet PDA.
    if ctx.accounts.rule_set_pda_info.data_is_empty() {
        create_or_allocate_account_raw(
            *program_id,
            ctx.accounts.rule_set_pda_info,
            ctx.accounts.system_program_info,
            ctx.accounts.payer_info,
            serialized_rule_set.len(),
            rule_set_seeds,
        )?;
    } else {
        resize_or_reallocate_account_raw(
            ctx.accounts.rule_set_pda_info,
            ctx.accounts.payer_info,
            ctx.accounts.system_program_info,
            serialized_rule_set.len(),
        )?;
    }

    // Copy user-pre-serialized RuleSet to PDA account.
    sol_memcpy(
        &mut ctx
            .accounts
            .rule_set_pda_info
            .try_borrow_mut_data()
            .unwrap(),
        &serialized_rule_set,
        serialized_rule_set.len(),
    );

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
    } = args;

    // If state is being updated for any Rules, the payer must be present and must be a signer so
    // that the RuleSet state PDA can be created or reallocated.
    if update_rule_state {
        if let Some(payer_info) = ctx.accounts.payer_info {
            if !payer_info.is_signer {
                return Err(RuleSetError::PayerIsNotSigner.into());
            }
        } else {
            return Err(ProgramError::NotEnoughAccountKeys);
        }
    }

    // RuleSet must be owned by this program.
    if *ctx.accounts.rule_set_pda_info.owner != crate::ID {
        return Err(RuleSetError::IncorrectOwner.into());
    }

    // RuleSet must not be empty.
    if ctx.accounts.rule_set_pda_info.data_is_empty() {
        return Err(RuleSetError::DataIsEmpty.into());
    }

    // Borrow the RuleSet PDA data.
    let data = ctx
        .accounts
        .rule_set_pda_info
        .data
        .try_borrow()
        .map_err(|_| RuleSetError::DataTypeMismatch)?;

    // Deserialize RuleSet.
    let rule_set: RuleSet =
        rmp_serde::from_slice(&data).map_err(|_| RuleSetError::MessagePackDeserializationError)?;

    // Check RuleSet account info derivation.
    let _bump = assert_derivation(
        program_id,
        ctx.accounts.rule_set_pda_info.key,
        &[
            PREFIX.as_bytes(),
            rule_set.owner().as_ref(),
            rule_set.name().as_bytes(),
        ],
    )?;

    // If RuleSet state is to be updated, check account info derivation.
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

    // Convert remaining Rule accounts into a map of `Pubkey`s to the corresponding `AccountInfo`s.
    // This makes it easy to pass the account infos into validation functions since they store the
    //`Pubkey`s.
    let accounts_map = ctx
        .remaining_accounts
        .iter()
        .map(|account| (*account.key, *account))
        .collect::<HashMap<Pubkey, &AccountInfo>>();

    // Get the Rule from the RuleSet based on the caller-specified operation.
    let rule = rule_set
        .get(operation)
        .ok_or(RuleSetError::OperationNotFound)?;

    // Validate the Rule.
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

pub fn cmp_pubkeys(a: &Pubkey, b: &Pubkey) -> bool {
    sol_memcmp(a.as_ref(), b.as_ref(), PUBKEY_BYTES) == 0
}
