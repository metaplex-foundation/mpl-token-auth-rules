use crate::{
    data::{AccountTag, Payload},
    error::RuleSetError,
    utils::assert_derivation,
};
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
        tags: &HashMap<AccountTag, Pubkey>,
        payloads: &HashMap<u8, Payload>,
    ) -> ProgramResult {
        match self {
            Rule::All { rules } => {
                msg!("Validating All");
                for rule in rules {
                    rule.validate(accounts, tags, payloads)?;
                }
                Ok(())
            }
            Rule::Any { rules } => {
                msg!("Validating Any");
                let mut error: Option<ProgramResult> = None;
                for rule in rules {
                    match rule.validate(accounts, tags, payloads) {
                        Ok(_) => return Ok(()),
                        Err(e) => error = Some(Err(e)),
                    }
                }
                error.unwrap_or_else(|| Err(RuleSetError::ErrorName.into()))
            }
            Rule::AdditionalSigner { account } => {
                msg!("Validating AdditionalSigner");
                if let Some(account) = accounts.get(account) {
                    if account.is_signer {
                        Ok(())
                    } else {
                        Err(RuleSetError::ErrorName.into())
                    }
                } else {
                    Err(RuleSetError::ErrorName.into())
                }
            }
            Rule::PubkeyMatch { account } => {
                msg!("Validating PubkeyMatch");
                if let Some(dest) = tags.get(&AccountTag::Destination) {
                    if dest == account {
                        Ok(())
                    } else {
                        Err(RuleSetError::ErrorName.into())
                    }
                } else {
                    Err(RuleSetError::ErrorName.into())
                }
            }
            Rule::DerivedKeyMatch { account } => {
                if let Some(Payload::DerivedKeyMatch { seeds }) = payloads.get(&self.to_u8()) {
                    if let Some(account) = accounts.get(account) {
                        let _bump = assert_derivation(&crate::id(), account, seeds)?;
                        Ok(())
                    } else {
                        Err(RuleSetError::ErrorName.into())
                    }
                } else {
                    Err(RuleSetError::ErrorName.into())
                }
            }
            Rule::ProgramOwned { program } => {
                msg!("Validating ProgramOwned");
                if let Some(account) = accounts.get(program) {
                    if account.owner == program {
                        Ok(())
                    } else {
                        Err(RuleSetError::ErrorName.into())
                    }
                } else {
                    Err(RuleSetError::ErrorName.into())
                }
            }
            Rule::Amount { amount } => {
                msg!("Validating Amount");
                if let Some(Payload::Amount { amount: a }) = payloads.get(&self.to_u8()) {
                    if amount == a {
                        Ok(())
                    } else {
                        Err(RuleSetError::ErrorName.into())
                    }
                } else {
                    Err(RuleSetError::ErrorName.into())
                }
            }
            Rule::Frequency { freq_account } => {
                // Deserialize the frequency account
                if let Some(account) = accounts.get(freq_account) {
                    let current_time = solana_program::clock::Clock::get()?.unix_timestamp;
                    let freq_account = FrequencyAccount::from_account_info(account);
                    if let Ok(freq_account) = freq_account {
                        if freq_account
                            .last_update
                            .checked_add(freq_account.period)
                            .unwrap()
                            <= current_time
                        {
                            Ok(())
                        } else {
                            Err(RuleSetError::ErrorName.into())
                        }
                    } else {
                        Err(RuleSetError::ErrorName.into())
                    }
                } else {
                    Err(RuleSetError::ErrorName.into())
                }
                // Grab the current time
                // Compare  last time + period to current time
            }
            Rule::PubkeyTreeMatch { root } => {
                msg!("Validating PubkeyTreeMatch");
                if let Some(Payload::PubkeyTreeMatch { proof, leaf }) = payloads.get(&self.to_u8())
                {
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
                        Err(RuleSetError::ErrorName.into())
                    }
                } else {
                    Err(RuleSetError::ErrorName.into())
                }
            }
        }
    }

    pub fn to_u8(&self) -> u8 {
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
