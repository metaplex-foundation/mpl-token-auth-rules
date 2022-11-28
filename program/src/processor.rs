use std::collections::HashMap;

use crate::{
    data::{AccountTag, Payload},
    instruction::RuleSetInstruction,
    pda::PREFIX,
    state::{Operation, Rule, RuleSet},
    utils::{assert_derivation, create_or_allocate_account_raw},
};
use borsh::BorshDeserialize;
use rmp_serde::{Deserializer, Serializer};
use serde::{Deserialize, Serialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
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
                // Convert the accounts into a map of Pubkeys to the corresponding account infos.
                // This makes it easy to pass the account infos into validation functions since they store the Pubkeys.
                let account_info_iter = &mut accounts.iter();

                let payer_info = next_account_info(account_info_iter)?;
                let ruleset_info = next_account_info(account_info_iter)?;
                let system_program_info = next_account_info(account_info_iter)?;

                let accounts_map = HashMap::from([
                    (*payer_info.key, payer_info),
                    (*ruleset_info.key, ruleset_info),
                    (*system_program_info.key, system_program_info),
                ]);

                // Tag accounts used across all rules with their use-case, such as destination, source, etc.
                let tags_map = HashMap::from([(AccountTag::Destination, *payer_info.key)]);

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

                let adtl_signer = Rule::AdditionalSigner {
                    account: *payer_info.key,
                };
                let adtl_signer2 = Rule::AdditionalSigner {
                    account: *payer_info.key,
                };
                let amount_check = Rule::Amount { amount: 1 };

                // Store the payloads that represent rule-specific data.
                let payloads_map =
                    HashMap::from([(amount_check.to_u8(), Payload::Amount { amount: 2 })]);

                let first_rule = Rule::All {
                    rules: vec![adtl_signer, adtl_signer2],
                };

                let overall_rule = Rule::All {
                    rules: vec![first_rule, amount_check],
                };

                let mut operations = RuleSet::new();
                operations.add(Operation::Transfer, overall_rule);

                //msg!("{:#?}", operations);

                // Serde
                //let serialized_data =
                //    serde_json::to_vec(&operations).map_err(|_| RuleSetError::ErrorName)?;

                // RMP serde
                let mut serialized_data = Vec::new();
                operations
                    .serialize(&mut Serializer::new(&mut serialized_data))
                    .unwrap();

                msg!("{:#?}", serialized_data);

                create_or_allocate_account_raw(
                    *program_id,
                    ruleset_info,
                    system_program_info,
                    payer_info,
                    serialized_data.len(),
                    ruleset_seeds,
                )?;

                // let unserialized_data: RuleSet =
                //     rmp_serde::from_slice(&serialized_data).map_err(|_| RuleSetError::ErrorName)?;

                //msg!("{:#?}", unserialized_data);

                let rule = operations.get(Operation::Transfer).unwrap();

                if let Ok(result) = rule.validate(&accounts_map, &tags_map, &payloads_map) {
                    msg!("{:#?}", result);
                } else {
                    msg!("Failed to validate");
                }

                Ok(())
            }
        }
    }
}
