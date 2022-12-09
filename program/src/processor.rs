use std::collections::HashMap;

use crate::{
    error::RuleSetError,
    instruction::RuleSetInstruction,
    pda::PREFIX,
    state::RuleSet,
    utils::{assert_derivation, create_or_allocate_account_raw},
};
use borsh::BorshDeserialize;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program_memory::sol_memcpy,
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
                let ruleset_pda_info = next_account_info(account_info_iter)?;
                let system_program_info = next_account_info(account_info_iter)?;

                if !payer_info.is_signer {
                    return Err(RuleSetError::PayerIsNotSigner.into());
                }

                // Check RuleSet account info derivation.
                let bump = assert_derivation(
                    program_id,
                    ruleset_pda_info,
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

                // Create or allocate RuleSet PDA account.
                create_or_allocate_account_raw(
                    *program_id,
                    ruleset_pda_info,
                    system_program_info,
                    payer_info,
                    args.serialized_rule_set.len(),
                    ruleset_seeds,
                )?;

                // Copy user-pre-serialized RuleSet to PDA account.
                sol_memcpy(
                    &mut ruleset_pda_info.try_borrow_mut_data().unwrap(),
                    &args.serialized_rule_set,
                    args.serialized_rule_set.len(),
                );

                Ok(())
            }
            RuleSetInstruction::Validate(args) => {
                let account_info_iter = &mut accounts.iter();
                let payer_info = next_account_info(account_info_iter)?;
                let ruleset_pda_info = next_account_info(account_info_iter)?;
                let _system_program_info = next_account_info(account_info_iter)?;

                if !payer_info.is_signer {
                    return Err(RuleSetError::PayerIsNotSigner.into());
                }

                // Check RuleSet account info derivation.
                let _bump = assert_derivation(
                    program_id,
                    ruleset_pda_info,
                    &[
                        PREFIX.as_bytes(),
                        payer_info.key.as_ref(),
                        args.name.as_bytes(),
                    ],
                )?;

                // Convert the accounts into a map of Pubkeys to the corresponding account infos.
                // This makes it easy to pass the account infos into validation functions since they store the Pubkeys.
                let accounts_map = accounts
                    .iter()
                    .map(|account| (*account.key, account))
                    .collect::<HashMap<Pubkey, &AccountInfo>>();

                // Borrow the RuleSet PDA data.
                let data = ruleset_pda_info
                    .data
                    .try_borrow()
                    .map_err(|_| RuleSetError::DataTypeMismatch)?;

                // Deserialize RuleSet.
                let rule_set: RuleSet =
                    rmp_serde::from_slice(&data).map_err(|_| RuleSetError::DataTypeMismatch)?;

                // Debug.
                msg!("{:#?}", rule_set);

                // Get the Rule from the RuleSet based on the caller-specified Operation.
                let rule = rule_set
                    .get(args.operation)
                    .ok_or(RuleSetError::DataTypeMismatch)?;

                // Validate the Rule.
                let result = rule.validate(&accounts_map, &args.payload);
                if !result.0 {
                    msg!("Failed to validate: {}", result.1);
                    return Err(result.1.into());
                }

                Ok(())
            }
        }
    }
}
