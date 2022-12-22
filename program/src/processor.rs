use std::collections::HashMap;

use crate::{
    error::RuleSetError,
    instruction::RuleSetInstruction,
    pda::{PREFIX, STATE_PDA},
    state::RuleSet,
    utils::{assert_derivation, create_or_allocate_account_raw},
    MAX_NAME_LENGTH,
};
use borsh::BorshDeserialize;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
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

                // Required accounts.
                let payer_info = next_account_info(account_info_iter)?;
                let rule_set_pda_info = next_account_info(account_info_iter)?;
                let system_program_info = next_account_info(account_info_iter)?;

                if !payer_info.is_signer {
                    return Err(RuleSetError::PayerIsNotSigner.into());
                }

                // Deserialize RuleSet.
                let rule_set: RuleSet = rmp_serde::from_slice(&args.serialized_rule_set)
                    .map_err(|_| RuleSetError::MessagePackDeserializationError)?;

                if rule_set.name().len() > MAX_NAME_LENGTH {
                    return Err(RuleSetError::NameTooLong.into());
                }

                // The payer/signer must be the RuleSet owner.
                if payer_info.key != rule_set.owner() {
                    return Err(RuleSetError::RuleSetOwnerMismatch.into());
                }

                // Check RuleSet account info derivation.
                let bump = assert_derivation(
                    program_id,
                    rule_set_pda_info.key,
                    &[
                        PREFIX.as_bytes(),
                        payer_info.key.as_ref(),
                        rule_set.name().as_bytes(),
                    ],
                )?;

                let rule_set_seeds = &[
                    PREFIX.as_ref(),
                    payer_info.key.as_ref(),
                    rule_set.name().as_ref(),
                    &[bump],
                ];

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

                // Required accounts.
                let rule_set_pda_info = next_account_info(account_info_iter)?;
                let mint_info = next_account_info(account_info_iter)?;
                let _system_program_info = next_account_info(account_info_iter)?;

                // Optional accounts are required if we are updating any Rule state.  Note that
                // `rule_authority_info is marked as unused here but this account is included below
                // in the `accounts_map` that is passed to Rule `validate`.
                let (payer_info, _rule_authority_info, rule_set_state_pda_info) =
                    if args.update_rule_state {
                        (
                            Some(next_account_info(account_info_iter)?),
                            Some(next_account_info(account_info_iter)?),
                            Some(next_account_info(account_info_iter)?),
                        )
                    } else {
                        (None, None, None)
                    };

                if args.update_rule_state {
                    if let Some(payer_info) = payer_info {
                        if !payer_info.is_signer {
                            return Err(RuleSetError::PayerIsNotSigner.into());
                        }
                    } else {
                        return Err(ProgramError::NotEnoughAccountKeys);
                    }
                }

                // RuleSet must be owned by this program.
                if *rule_set_pda_info.owner != crate::ID {
                    return Err(RuleSetError::IncorrectOwner.into());
                }

                // RuleSet must not be empty.
                if rule_set_pda_info.data_is_empty() {
                    return Err(RuleSetError::DataIsEmpty.into());
                }

                // Borrow the RuleSet PDA data.
                let data = rule_set_pda_info
                    .data
                    .try_borrow()
                    .map_err(|_| RuleSetError::DataTypeMismatch)?;

                // Deserialize RuleSet.
                let rule_set: RuleSet = rmp_serde::from_slice(&data)
                    .map_err(|_| RuleSetError::MessagePackDeserializationError)?;

                // Check RuleSet account info derivation.
                let _bump = assert_derivation(
                    program_id,
                    rule_set_pda_info.key,
                    &[
                        PREFIX.as_bytes(),
                        rule_set.owner().as_ref(),
                        rule_set.name().as_bytes(),
                    ],
                )?;

                // If RuleSet state is to be updated, check account info derivation.
                if args.update_rule_state {
                    if let Some(rule_set_state_pda_info) = rule_set_state_pda_info {
                        let _bump = assert_derivation(
                            program_id,
                            rule_set_state_pda_info.key,
                            &[
                                STATE_PDA.as_bytes(),
                                rule_set.owner().as_ref(),
                                rule_set.name().as_bytes(),
                                mint_info.key.as_ref(),
                            ],
                        )?;
                    } else {
                        return Err(ProgramError::NotEnoughAccountKeys);
                    }
                }

                // Convert the accounts into a map of `Pubkey`s to the corresponding account infos.
                // This makes it easy to pass the account infos into validation functions since
                // they store the `Pubkey`s.
                let accounts_map = accounts
                    .iter()
                    .map(|account| (*account.key, account))
                    .collect::<HashMap<Pubkey, &AccountInfo>>();

                // Get the Rule from the RuleSet based on the caller-specified operation.
                let rule = rule_set
                    .get(args.operation)
                    .ok_or(RuleSetError::OperationNotFound)?;

                // Validate the Rule.
                if let Err(err) = rule.validate(
                    &accounts_map,
                    &args.payload,
                    args.update_rule_state,
                    rule_set_state_pda_info.map(|account_info| account_info.key),
                ) {
                    msg!("Failed to validate: {}", err);
                    return Err(err);
                }

                Ok(())
            }
        }
    }
}
