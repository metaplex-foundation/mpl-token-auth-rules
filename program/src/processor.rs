use std::collections::HashMap;

use crate::{
    error::RuleSetError,
    instruction::RuleSetInstruction,
    pda::PREFIX,
    state::{primitives::Validation, rules::rule_set::RuleSet, Operation},
    utils::{assert_derivation, create_or_allocate_account_raw},
};
use borsh::BorshDeserialize;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    pubkey::Pubkey,
};

use rmp_serde::{Deserializer, Serializer};
use serde::{Deserialize, Serialize};

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

                let accounts_map = HashMap::from([
                    (*payer_info.key, payer_info),
                    (*ruleset_info.key, ruleset_info),
                    (*system_program_info.key, system_program_info),
                ]);

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

                let adtl_signer = Validation::AdditionalSigner {
                    account: *payer_info.key,
                };
                let adtl_signer2 = Validation::AdditionalSigner {
                    account: *payer_info.key,
                };
                let adtl_signer3 = Validation::AdditionalSigner {
                    account: *payer_info.key,
                };

                let first_rule = Validation::All {
                    validations: vec![adtl_signer, adtl_signer2],
                };

                let overall_rule = Validation::Any {
                    validations: vec![first_rule, adtl_signer3],
                };

                let mut operations = RuleSet::new();
                operations.add(Operation::Transfer, overall_rule);

                msg!("{:#?}", operations);

                // Serde
                //let serialized_data =
                //    serde_json::to_vec(&operations).map_err(|_| RuleSetError::ErrorName)?;

                // RMP serde
                let mut serialized_data = Vec::new();
                operations
                    .serialize(&mut Serializer::new(&mut serialized_data))
                    .unwrap();

                // create_or_allocate_account_raw(
                //     *program_id,
                //     ruleset_info,
                //     system_program_info,
                //     payer_info,
                //     serialized_data.len(),
                //     ruleset_seeds,
                // )?;

                let unserialized_data: RuleSet =
                    rmp_serde::from_slice(&serialized_data).map_err(|_| RuleSetError::ErrorName)?;

                msg!("{:#?}", unserialized_data);

                let validation = operations.get(Operation::Transfer).unwrap();

                msg!(
                    "Rule validation result: {}",
                    validation.validate(&accounts_map)
                );

                Ok(())
            }
        }
    }
}
