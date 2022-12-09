use std::collections::HashMap;

use crate::{
    error::RuleSetError,
    instruction::RuleSetInstruction,
    pda::{FREQ_PDA, PREFIX},
    state::RuleSet,
    utils::{assert_derivation, create_or_allocate_account_raw},
};
use borsh::{BorshDeserialize, BorshSerialize};
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
                let rule_set_pda_info = next_account_info(account_info_iter)?;
                let system_program_info = next_account_info(account_info_iter)?;

                if !payer_info.is_signer {
                    return Err(RuleSetError::PayerIsNotSigner.into());
                }

                // Check RuleSet account info derivation.
                let bump = assert_derivation(
                    program_id,
                    rule_set_pda_info.key,
                    &[
                        PREFIX.as_bytes(),
                        payer_info.key.as_ref(),
                        args.rule_set_name.as_bytes(),
                    ],
                )?;

                let rule_set_seeds = &[
                    PREFIX.as_ref(),
                    payer_info.key.as_ref(),
                    args.rule_set_name.as_ref(),
                    &[bump],
                ];

                // Deserialize RuleSet.
                let rule_set: RuleSet = rmp_serde::from_slice(&args.serialized_rule_set)
                    .map_err(|_| RuleSetError::DataTypeMismatch)?;

                // Validate any PDA derivations present in the RuleSet.
                for (_operation, rule) in rule_set.operations {
                    rule.assert_rule_pda_derivations(payer_info.key, &args.rule_set_name)?;
                }

                // Create or allocate RuleSet PDA account.
                create_or_allocate_account_raw(
                    *program_id,
                    rule_set_pda_info,
                    system_program_info,
                    payer_info,
                    args.serialized_rule_set.len(),
                    rule_set_seeds,
                )?;

                // Copy user-pre-serialized RuleSet to PDA account.
                sol_memcpy(
                    &mut rule_set_pda_info.try_borrow_mut_data().unwrap(),
                    &args.serialized_rule_set,
                    args.serialized_rule_set.len(),
                );

                Ok(())
            }
            RuleSetInstruction::Validate(args) => {
                let account_info_iter = &mut accounts.iter();
                let payer_info = next_account_info(account_info_iter)?;
                let rule_set_pda_info = next_account_info(account_info_iter)?;
                let _system_program_info = next_account_info(account_info_iter)?;

                if !payer_info.is_signer {
                    return Err(RuleSetError::PayerIsNotSigner.into());
                }

                // Check RuleSet account info derivation.
                let _bump = assert_derivation(
                    program_id,
                    rule_set_pda_info.key,
                    &[
                        PREFIX.as_bytes(),
                        payer_info.key.as_ref(),
                        args.rule_set_name.as_bytes(),
                    ],
                )?;

                // Convert the accounts into a map of Pubkeys to the corresponding account infos.
                // This makes it easy to pass the account infos into validation functions since they store the Pubkeys.
                let accounts_map = accounts
                    .iter()
                    .map(|account| (*account.key, account))
                    .collect::<HashMap<Pubkey, &AccountInfo>>();

                // Borrow the RuleSet PDA data.
                let data = rule_set_pda_info
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
                if let Err(err) = rule.validate(&accounts_map, &args.payload) {
                    msg!("Failed to validate: {}", err);
                    return Err(err);
                }

                Ok(())
            }
            RuleSetInstruction::CreateFrequencyRule(args) => {
                let account_info_iter = &mut accounts.iter();
                let payer_info = next_account_info(account_info_iter)?;
                let freq_rule_pda_info = next_account_info(account_info_iter)?;
                let system_program_info = next_account_info(account_info_iter)?;

                if !payer_info.is_signer {
                    return Err(RuleSetError::PayerIsNotSigner.into());
                }

                // Check Frequency PDA account info derivation.
                let bump = assert_derivation(
                    program_id,
                    freq_rule_pda_info.key,
                    &[
                        FREQ_PDA.as_bytes(),
                        payer_info.key.as_ref(),
                        args.rule_set_name.as_bytes(),
                        args.freq_rule_name.as_bytes(),
                    ],
                )?;

                let freq_pda_seeds = &[
                    FREQ_PDA.as_bytes(),
                    payer_info.key.as_ref(),
                    args.rule_set_name.as_bytes(),
                    args.freq_rule_name.as_bytes(),
                    &[bump],
                ];

                // Serialize the Frequency Rule.
                let serialized_rule = args
                    .freq_data
                    .try_to_vec()
                    .map_err(|_| RuleSetError::BorshSerializationError)?;

                // Create or allocate Frequency PDA account.
                create_or_allocate_account_raw(
                    *program_id,
                    freq_rule_pda_info,
                    system_program_info,
                    payer_info,
                    serialized_rule.len(),
                    freq_pda_seeds,
                )?;

                // Copy Frequency Rule to PDA account.
                sol_memcpy(
                    &mut freq_rule_pda_info.try_borrow_mut_data().unwrap(),
                    &serialized_rule,
                    serialized_rule.len(),
                );

                Ok(())
            }
        }
    }
}
