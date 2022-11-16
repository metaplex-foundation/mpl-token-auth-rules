use crate::{
    error::RuleSetError,
    instruction::RuleSetInstruction,
    pda::PREFIX,
    state::rules::rule_set::RuleSet,
    utils::{assert_derivation, create_or_allocate_account_raw},
};
use borsh::BorshDeserialize;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    pubkey::Pubkey,
};

pub struct Processor;
impl Processor {
    pub fn process_instruction(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction = RuleSetInstruction::try_from_slice(instruction_data)?;
        match instruction {
            RuleSetInstruction::Create(args) => {
                let account_info_iter = &mut accounts.iter();

                let payer_info = next_account_info(account_info_iter)?;
                let ruleset_info = next_account_info(account_info_iter)?;
                let system_program_info = next_account_info(account_info_iter)?;

                let bump = assert_derivation(
                    program_id,
                    ruleset_info,
                    &[
                        PREFIX.as_bytes(),
                        payer_info.key.as_ref(),
                        args.name.as_bytes(),
                    ],
                )?;

                let ruleset_seeds = &[
                    PREFIX.as_ref(),
                    payer_info.key.as_ref(),
                    args.name.as_ref(),
                    &[bump],
                ];

                let mut operations = RuleSet::new();

                let serialized_data = operations
                    .try_to_vec()
                    .map_err(|_| RuleSetError::ErrorName)?;

                create_or_allocate_account_raw(
                    *program_id,
                    ruleset_info,
                    system_program_info,
                    payer_info,
                    serialized_data.len(),
                    ruleset_seeds,
                );
                Ok(())
            }
        }
    }
}
