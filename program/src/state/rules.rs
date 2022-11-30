use crate::{error::RuleSetError, utils::assert_derivation, Payload, PayloadVec};
use serde::{Deserialize, Serialize};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, pubkey::Pubkey, sysvar::Sysvar,
};
use std::collections::HashMap;

use super::{FrequencyAccount, SolanaAccount};

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub enum Rule {
    All { rules: Vec<Rule> },
    Any { rules: Vec<Rule> },
    AdditionalSigner { account: Pubkey },
    PubkeyMatch { account: Pubkey },
    DerivedKeyMatch { account: Pubkey },
    ProgramOwned { program: Pubkey },
    Amount { amount: u64 },
    Frequency { freq_account: Pubkey },
    PubkeyTreeMatch { root: [u8; 32] },
}

impl Rule {
    pub fn validate(
        &self,
        accounts: &HashMap<Pubkey, &AccountInfo>,
        payloads: &PayloadVec,
    ) -> ProgramResult {
        match self {
            Rule::All { rules } => {
                msg!("Validating All");
                for rule in rules {
                    rule.validate(accounts, payloads)?;
                }
                Ok(())
            }
            Rule::Any { rules } => {
                msg!("Validating Any");
                let mut error: Option<ProgramResult> = None;
                for rule in rules {
                    match rule.validate(accounts, payloads) {
                        Ok(_) => return Ok(()),
                        Err(e) => error = Some(Err(e)),
                    }
                }
                error.unwrap_or_else(|| Err(RuleSetError::DataTypeMismatch.into()))
            }
            Rule::AdditionalSigner { account } => {
                msg!("Validating AdditionalSigner");
                if let Some(account) = accounts.get(account) {
                    if account.is_signer {
                        Ok(())
                    } else {
                        Err(RuleSetError::AdditionalSignerCheckFailed.into())
                    }
                } else {
                    Err(RuleSetError::AdditionalSignerCheckFailed.into())
                }
            }
            Rule::PubkeyMatch { account } => {
                msg!("Validating PubkeyMatch");
                if let Some(Payload::PubkeyMatch { destination: d }) = payloads.get(self) {
                    if d == account {
                        Ok(())
                    } else {
                        Err(RuleSetError::PubkeyMatchCheckFailed.into())
                    }
                } else {
                    Err(RuleSetError::PubkeyMatchCheckFailed.into())
                }
            }
            Rule::DerivedKeyMatch { account } => {
                msg!("Validating DerivedKeyMatch");
                if let Some(Payload::DerivedKeyMatch { seeds }) = payloads.get(self) {
                    if let Some(account) = accounts.get(account) {
                        let vec_of_slices = seeds.iter().map(Vec::as_slice).collect::<Vec<&[u8]>>();
                        let seeds = &vec_of_slices[..];
                        let _bump = assert_derivation(&crate::id(), account, seeds)?;
                        Ok(())
                    } else {
                        Err(RuleSetError::DerivedKeyMatchCheckFailed.into())
                    }
                } else {
                    Err(RuleSetError::DerivedKeyMatchCheckFailed.into())
                }
            }
            Rule::ProgramOwned { program } => {
                msg!("Validating ProgramOwned");
                if let Some(account) = accounts.get(program) {
                    if account.owner == program {
                        Ok(())
                    } else {
                        Err(RuleSetError::ProgramOwnedCheckFailed.into())
                    }
                } else {
                    Err(RuleSetError::ProgramOwnedCheckFailed.into())
                }
            }
            Rule::Amount { amount } => {
                msg!("Validating Amount");
                if let Some(Payload::Amount { amount: a }) = payloads.get(self) {
                    if amount == a {
                        Ok(())
                    } else {
                        Err(RuleSetError::AmountCheckFailed.into())
                    }
                } else {
                    Err(RuleSetError::AmountCheckFailed.into())
                }
            }
            Rule::Frequency { freq_account } => {
                msg!("Validating Frequency");
                // Deserialize the frequency account
                if let Some(account) = accounts.get(freq_account) {
                    let current_time = solana_program::clock::Clock::get()?.unix_timestamp;
                    let freq_account = FrequencyAccount::from_account_info(account);
                    if let Ok(freq_account) = freq_account {
                        if freq_account
                            .last_update
                            .checked_add(freq_account.period)
                            .ok_or(RuleSetError::NumericalOverflow)?
                            <= current_time
                        {
                            Ok(())
                        } else {
                            Err(RuleSetError::FrequencyCheckFailed.into())
                        }
                    } else {
                        Err(RuleSetError::FrequencyCheckFailed.into())
                    }
                } else {
                    Err(RuleSetError::FrequencyCheckFailed.into())
                }
                // Grab the current time
                // Compare  last time + period to current time
            }
            Rule::PubkeyTreeMatch { root } => {
                msg!("Validating PubkeyTreeMatch");
                if let Some(Payload::PubkeyTreeMatch { proof, leaf }) = payloads.get(self) {
                    let mut computed_hash = *leaf;
                    for proof_element in proof.iter() {
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
                        Ok(())
                    } else {
                        Err(RuleSetError::PubkeyTreeMatchCheckFailed.into())
                    }
                } else {
                    Err(RuleSetError::PubkeyTreeMatchCheckFailed.into())
                }
            }
        }
    }

    pub fn to_usize(&self) -> usize {
        match self {
            Rule::All { .. } => 0,
            Rule::Any { .. } => 1,
            Rule::AdditionalSigner { .. } => 2,
            Rule::PubkeyMatch { .. } => 3,
            Rule::DerivedKeyMatch { .. } => 4,
            Rule::ProgramOwned { .. } => 5,
            Rule::Amount { .. } => 6,
            Rule::Frequency { .. } => 7,
            Rule::PubkeyTreeMatch { .. } => 8,
        }
    }
}
