use std::collections::HashMap;

use crate::{
    error::RuleSetError,
    instruction::RuleSetInstruction,
    pda::PREFIX,
    state::{Operation, Rule, RuleSet},
    utils::{assert_derivation, create_or_allocate_account_raw},
    Payload, PayloadVec,
};
use borsh::BorshDeserialize;
use rmp_serde::Serializer;
use serde::Serialize;
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

                let adtl_signer = Rule::AdditionalSigner {
                    account: *payer_info.key,
                };
                let adtl_signer2 = Rule::AdditionalSigner {
                    account: *payer_info.key,
                };
                let amount_check = Rule::Amount { amount: 1 };

                let first_rule = Rule::All {
                    rules: vec![adtl_signer, adtl_signer2],
                };

                let overall_rule = Rule::All {
                    rules: vec![first_rule, amount_check],
                };

                let mut operations = RuleSet::new();
                operations.add(Operation::Transfer, overall_rule);

                let mut serialized_rule_set = Vec::new();
                operations
                    .serialize(&mut Serializer::new(&mut serialized_rule_set))
                    .unwrap();

                // Create or allocate RuleSet PDA account.
                create_or_allocate_account_raw(
                    *program_id,
                    ruleset_pda_info,
                    system_program_info,
                    payer_info,
                    serialized_rule_set.len(),
                    ruleset_seeds,
                )?;

                // Copy user-pre-serialized RuleSet to PDA account.
                sol_memcpy(
                    &mut **ruleset_pda_info.try_borrow_mut_data().unwrap(),
                    &serialized_rule_set,
                    serialized_rule_set.len(),
                );

                msg!("{:#?}", serialized_rule_set == args.serialized_rule_set);

                let unserialized_data: RuleSet = rmp_serde::from_slice(&args.serialized_rule_set)
                    .map_err(|_| RuleSetError::ErrorName)?;

                msg!("{:#?}", unserialized_data);

                // Get the Rule from the RuleSet based on the caller-specified Operation.
                let rule = unserialized_data
                    .get(Operation::Transfer)
                    .ok_or(RuleSetError::ErrorName)?;

                let accounts_map = HashMap::from([
                    (*payer_info.key, payer_info),
                    (*ruleset_pda_info.key, ruleset_pda_info),
                    (*system_program_info.key, system_program_info),
                ]);

                // Store the payloads that represent rule-specific data.
                let mut payloads_vec = PayloadVec::new();
                payloads_vec.add(&(Rule::Amount { amount: 1 }), Payload::Amount { amount: 1 })?;
                // HashMap::from([(amount_check.to_u8(), Payload::Amount { amount: 2 })]);

                // Validate the Rule.
                if let Err(err) = rule.validate(&accounts_map, &payloads_vec) {
                    msg!("Failed to validate: {}", err);
                    return Err(err);
                }

                Ok(())
            }
            RuleSetInstruction::Validate(args) => {
                let account_info_iter = &mut accounts.iter();
                let payer_info = next_account_info(account_info_iter)?;
                let ruleset_pda_info = next_account_info(account_info_iter)?;
                let _system_program_info = next_account_info(account_info_iter)?;

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
                    .map_err(|_| RuleSetError::ErrorName)?;

                // Deserialize RuleSet.
                let rule_set: RuleSet =
                    rmp_serde::from_slice(&data).map_err(|_| RuleSetError::ErrorName)?;

                // Debug.
                msg!("{:#?}", rule_set);

                // Get the Rule from the RuleSet based on the caller-specified Operation.
                let rule = rule_set
                    .get(args.operation)
                    .ok_or(RuleSetError::ErrorName)?;

                // Validate the Rule.
                if let Err(err) = rule.validate(&accounts_map, &args.payloads) {
                    msg!("Failed to validate: {}", err);
                    return Err(err);
                }

                Ok(())
            }
        }
    }
}
