use crate::{
    error::RuleSetError,
    instruction::RuleSetInstruction,
    pda::PREFIX,
    state::{primitives::Validation, rules::rule_set::RuleSet},
    utils::{assert_derivation, create_or_allocate_account_raw},
};
use borsh::{BorshDeserialize, BorshSerialize};
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

                let adtl_signer = Validation::AdditionalSigner {
                    account: *payer_info.key,
                };
                let adtl_signer2 = Validation::AdditionalSigner {
                    account: *payer_info.key,
                };
                let adtl_signer3 = Validation::AdditionalSigner {
                    account: *payer_info.key,
                };

                // let accounts_map =
                //     HashMap::from([(*acc1.key, &acc1), (*acc2.key, &acc2), (*acc3.key, &acc3)]);

                let first_rule =
                    Validation::All(vec![Box::new(adtl_signer), Box::new(adtl_signer2)]);

                let overall_rule =
                    Validation::Any(vec![Box::new(first_rule), Box::new(adtl_signer3)]);

                let mut operations = RuleSet::new();

                // let serialized_data = operations
                //     .try_to_vec()
                //     .map_err(|_| RuleSetError::ErrorName)?;
                msg!("{:#?}", operations);

                let serialized_data =
                    serde_json::to_vec(&operations).map_err(|_| RuleSetError::ErrorName)?;

                create_or_allocate_account_raw(
                    *program_id,
                    ruleset_info,
                    system_program_info,
                    payer_info,
                    serialized_data.len(),
                    ruleset_seeds,
                )?;
                Ok(())
            }
        }
    }
}
