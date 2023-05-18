use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_memory::sol_memcpy,
    pubkey::Pubkey,
};

use crate::{
    error::RuleSetError,
    instruction::{Context, WriteToBuffer, WriteToBufferArgs},
    pda::PREFIX,
    utils::{assert_derivation, create_or_allocate_account_raw, resize_or_reallocate_account_raw},
};

// Function to match on `WriteToBuffer` version and call correct implementation.
pub(crate) fn write_to_buffer<'a>(
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
