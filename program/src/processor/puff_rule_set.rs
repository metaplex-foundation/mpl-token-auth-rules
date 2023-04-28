use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};

use crate::{
    error::RuleSetError,
    instruction::{Context, PuffRuleSet, PuffRuleSetArgs},
    pda::PREFIX,
    state::CHUNK_SIZE,
    utils::{assert_derivation, create_or_allocate_account_raw, resize_or_reallocate_account_raw},
};

// Function to match on `PuffRuleSet` version and call correct implementation.
pub(crate) fn puff_rule_set<'a>(
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
