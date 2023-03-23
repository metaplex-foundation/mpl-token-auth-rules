//! The processors for the Rule Set program instructions.
//!
//! See state module for description of PDA memory layout.

mod create_or_update;
mod puff_rule_set;
mod validate;
mod write_to_buffer;

use borsh::BorshDeserialize;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::{
    instruction::RuleSetInstruction,
    processor::{
        create_or_update::create_or_update, puff_rule_set::puff_rule_set, validate::validate,
        write_to_buffer::write_to_buffer,
    },
    utils::cmp_pubkeys,
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
