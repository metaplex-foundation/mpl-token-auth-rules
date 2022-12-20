use crate::{
    error::RuleSetError,
    payload::{Payload, PayloadKey},
    pda::FREQ_PDA,
    utils::assert_derivation,
};
use serde::{Deserialize, Serialize};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey, sysvar::Sysvar,
};
use std::collections::HashMap;

use super::{FrequencyAccount, SolanaAccount};

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub enum Rule {
    All {
        rules: Vec<Rule>,
    },
    Any {
        rules: Vec<Rule>,
    },
    Not {
        rule: Box<Rule>,
    },
    AdditionalSigner {
        account: Pubkey,
    },
    PubkeyMatch {
        pubkey: Pubkey,
        field: PayloadKey,
    },
    DerivedKeyMatch {
        account: Pubkey,
        field: PayloadKey,
    },
    ProgramOwned {
        program: Pubkey,
        field: PayloadKey,
    },
    Amount {
        amount: u64,
    },
    Frequency {
        freq_name: String,
        freq_account: Pubkey,
    },
    PubkeyTreeMatch {
        root: [u8; 32],
        field: PayloadKey,
    },
}

impl Rule {
    pub fn validate(
        &self,
        accounts: &HashMap<Pubkey, &AccountInfo>,
        payload: &Payload,
        update_rule_state: bool,
    ) -> ProgramResult {
        let (status, rollup_err) = self.low_level_validate(accounts, payload, update_rule_state);

        if status {
            ProgramResult::Ok(())
        } else {
            ProgramResult::Err(rollup_err)
        }
    }

    pub fn low_level_validate(
        &self,
        accounts: &HashMap<Pubkey, &AccountInfo>,
        payload: &Payload,
        update_rule_state: bool,
    ) -> (bool, ProgramError) {
        match self {
            Rule::All { rules } => {
                msg!("Validating All");
                let mut last = self.to_error();
                for rule in rules {
                    last = rule.to_error();
                    let result = rule.low_level_validate(accounts, payload, update_rule_state);
                    if !result.0 {
                        return result;
                    }
                }
                (true, last)
            }
            Rule::Any { rules } => {
                msg!("Validating Any");
                let mut last = self.to_error();
                for rule in rules {
                    last = rule.to_error();
                    let result = rule.low_level_validate(accounts, payload, update_rule_state);
                    if result.0 {
                        return result;
                    }
                }
                (false, last)
            }
            Rule::Not { rule } => {
                let result = rule.low_level_validate(accounts, payload, update_rule_state);
                (!result.0, result.1)
            }
            Rule::AdditionalSigner { account } => {
                msg!("Validating AdditionalSigner");
                if let Some(signer) = accounts.get(account) {
                    (signer.is_signer, self.to_error())
                } else {
                    (false, RuleSetError::MissingAccount.into())
                }
            }
            Rule::PubkeyMatch { pubkey, field } => {
                msg!("Validating PubkeyMatch");

                let key = match payload.get_pubkey(field) {
                    Some(pubkey) => pubkey,
                    _ => return (false, RuleSetError::MissingPayloadValue.into()),
                };

                if key == pubkey {
                    (true, self.to_error())
                } else {
                    (false, self.to_error())
                }
            }
            Rule::DerivedKeyMatch { account, field } => {
                msg!("Validating DerivedKeyMatch");

                let seeds = match payload.get_seeds(field) {
                    Some(seeds) => seeds,
                    _ => return (false, RuleSetError::MissingPayloadValue.into()),
                };

                let vec_of_slices = seeds
                    .seeds
                    .iter()
                    .map(Vec::as_slice)
                    .collect::<Vec<&[u8]>>();
                let seeds = &vec_of_slices[..];
                if let Ok(_bump) = assert_derivation(&crate::ID, account, seeds) {
                    (true, self.to_error())
                } else {
                    (false, self.to_error())
                }
            }
            Rule::ProgramOwned { program, field } => {
                msg!("Validating ProgramOwned");

                let key = match payload.get_pubkey(field) {
                    Some(pubkey) => pubkey,
                    _ => return (false, RuleSetError::MissingPayloadValue.into()),
                };

                if let Some(account) = accounts.get(key) {
                    if *account.owner == *program {
                        return (true, self.to_error());
                    }
                } else {
                    return (false, RuleSetError::MissingAccount.into());
                }

                (false, self.to_error())
            }
            Rule::Amount { amount } => {
                msg!("Validating Amount");
                if let Some(payload_amount) = &payload.get_amount(&PayloadKey::Amount) {
                    if amount == payload_amount {
                        (true, self.to_error())
                    } else {
                        (false, self.to_error())
                    }
                } else {
                    (false, RuleSetError::MissingPayloadValue.into())
                }
            }
            Rule::Frequency {
                freq_name: _,
                freq_account,
            } => {
                msg!("Validating Frequency");
                // Get the Frequency Rule `AccountInfo`.
                let freq_account_info = if let Some(account_info) = accounts.get(freq_account) {
                    account_info
                } else {
                    return (false, RuleSetError::MissingAccount.into());
                };

                // Grab the current time.
                let current_time = match solana_program::clock::Clock::get() {
                    Ok(clock) => clock,
                    Err(err) => return (false, err),
                };

                // Deserialize the Frequency Rule account data.
                let mut freq_account_data =
                    match FrequencyAccount::from_account_info(freq_account_info) {
                        Ok(freq_account_data) => freq_account_data,
                        Err(err) => return (false, err),
                    };

                // Calculate last time + period.
                let freq_check = if let Some(val) = freq_account_data
                    .last_update
                    .checked_add(freq_account_data.period)
                {
                    val
                } else {
                    return (false, RuleSetError::NumericalOverflow.into());
                };

                // Compare current time to last time + period.
                if current_time.unix_timestamp < freq_check {
                    return (false, self.to_error());
                }

                // If requested, update `last_update` time in Frequency rule to current time.
                if update_rule_state {
                    freq_account_data.last_update = current_time.unix_timestamp;
                    // Serialize the Frequency Rule.
                    match freq_account_data.to_account_data(freq_account_info) {
                        Ok(_) => (true, self.to_error()),
                        Err(err) => (false, err),
                    }
                } else {
                    (true, self.to_error())
                }
            }
            Rule::PubkeyTreeMatch { root, field } => {
                msg!("Validating PubkeyTreeMatch");

                let merkle_proof = match payload.get_merkle_proof(field) {
                    Some(merkle_proof) => merkle_proof,
                    _ => return (false, RuleSetError::MissingPayloadValue.into()),
                };

                let mut computed_hash = merkle_proof.leaf;
                for proof_element in merkle_proof.proof.iter() {
                    if computed_hash <= *proof_element {
                        // Hash(current computed hash + current element of the proof)
                        computed_hash = solana_program::keccak::hashv(&[
                            &[0x01],
                            &computed_hash,
                            proof_element,
                        ])
                        .0;
                    } else {
                        // Hash(current element of the proof + current computed hash)
                        computed_hash = solana_program::keccak::hashv(&[
                            &[0x01],
                            proof_element,
                            &computed_hash,
                        ])
                        .0;
                    }
                }
                // Check if the computed hash (root) is equal to the provided root
                if computed_hash == *root {
                    (true, self.to_error())
                } else {
                    (false, self.to_error())
                }
            }
        }
    }

    pub fn assert_rule_pda_derivations(
        &self,
        owner: &Pubkey,
        rule_set_name: &String,
        accounts: &HashMap<Pubkey, &AccountInfo>,
    ) -> ProgramResult {
        match self {
            Rule::All { rules } => {
                for rule in rules {
                    rule.assert_rule_pda_derivations(owner, rule_set_name, accounts)?;
                }
                Ok(())
            }
            Rule::Any { rules } => {
                let mut error: Option<ProgramResult> = None;
                for rule in rules {
                    match rule.assert_rule_pda_derivations(owner, rule_set_name, accounts) {
                        Ok(_) => return Ok(()),
                        Err(e) => error = Some(Err(e)),
                    }
                }
                error.unwrap_or_else(|| Err(RuleSetError::DataTypeMismatch.into()))
            }
            Rule::Frequency {
                freq_name,
                freq_account,
            } => {
                msg!("Assert Frequency PDA deriviation");
                if let Some(freq_pda_account_info) = accounts.get(freq_account) {
                    // Frequency PDA account must be owned by this program.
                    if *freq_pda_account_info.owner != crate::ID {
                        return Err(RuleSetError::IncorrectOwner.into());
                    }

                    // Frequency PDA account must not be empty.
                    if freq_pda_account_info.data_is_empty() {
                        return Err(RuleSetError::DataIsEmpty.into());
                    }

                    // Check Frequency account info derivation.
                    let _bump = assert_derivation(
                        &crate::ID,
                        freq_account,
                        &[
                            FREQ_PDA.as_bytes(),
                            owner.as_ref(),
                            rule_set_name.as_bytes(),
                            freq_name.as_bytes(),
                        ],
                    )?;
                } else {
                    return Err(RuleSetError::MissingAccount.into());
                }

                Ok(())
            }
            _ => Ok(()),
        }
    }

    pub fn to_error(&self) -> ProgramError {
        match self {
            Rule::AdditionalSigner { .. } => RuleSetError::AdditionalSignerCheckFailed.into(),
            Rule::PubkeyMatch { .. } => RuleSetError::PubkeyMatchCheckFailed.into(),
            Rule::DerivedKeyMatch { .. } => RuleSetError::DerivedKeyMatchCheckFailed.into(),
            Rule::ProgramOwned { .. } => RuleSetError::ProgramOwnedCheckFailed.into(),
            Rule::Amount { .. } => RuleSetError::AmountCheckFailed.into(),
            Rule::Frequency { .. } => RuleSetError::FrequencyCheckFailed.into(),
            Rule::PubkeyTreeMatch { .. } => RuleSetError::PubkeyTreeMatchCheckFailed.into(),
            _ => RuleSetError::NotImplemented.into(),
        }
    }
}
