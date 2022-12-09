use crate::{
    error::RuleSetError,
    payload::{LeafInfo, SeedsVec},
    utils::assert_derivation,
    Payload,
};
use serde::{Deserialize, Serialize};
use solana_program::{account_info::AccountInfo, msg, pubkey::Pubkey, sysvar::Sysvar};
use std::collections::HashMap;

use super::{FrequencyAccount, SolanaAccount};

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub enum Rule {
    All { rules: Vec<Rule> },
    Any { rules: Vec<Rule> },
    Not { rule: Box<Rule> },
    AdditionalSigner { account: Pubkey },
    PubkeyMatch { destination: Pubkey },
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
        payload: &Payload,
    ) -> (bool, usize) {
        match self {
            Rule::All { rules } => {
                msg!("Validating All");
                let mut last = self.to_usize();
                for rule in rules {
                    last = rule.to_usize();
                    let result = rule.validate(accounts, payload);
                    if !result.0 {
                        return result;
                    }
                }
                (true, last)
            }
            Rule::Any { rules } => {
                msg!("Validating Any");
                let mut last = self.to_usize();
                for rule in rules {
                    last = rule.to_usize();
                    let result = rule.validate(accounts, payload);
                    if result.0 {
                        return result;
                    }
                }
                (false, last)
            }
            Rule::Not { rule } => {
                let result = rule.validate(accounts, payload);
                (!result.0, result.1)
            }
            Rule::AdditionalSigner { account } => {
                msg!("Validating AdditionalSigner");
                if let Some(signer) = accounts.get(account) {
                    (signer.is_signer, self.to_usize())
                } else {
                    (false, self.to_usize())
                }
            }
            Rule::PubkeyMatch { destination } => {
                msg!("Validating PubkeyMatch");
                if let Some(payload_destination) = &payload.destination_key {
                    if destination == payload_destination {
                        (true, self.to_usize())
                    } else {
                        (false, self.to_usize())
                    }
                } else {
                    (false, self.to_usize())
                }
            }
            Rule::DerivedKeyMatch { account } => {
                msg!("Validating DerivedKeyMatch");
                if let Some(SeedsVec { seeds }) = &payload.derived_key_seeds {
                    if let Some(account) = accounts.get(account) {
                        let vec_of_slices = seeds.iter().map(Vec::as_slice).collect::<Vec<&[u8]>>();
                        let seeds = &vec_of_slices[..];
                        if let Ok(_bump) = assert_derivation(&crate::id(), account, seeds) {
                            (true, self.to_usize())
                        } else {
                            (false, self.to_usize())
                        }
                    } else {
                        (false, self.to_usize())
                    }
                } else {
                    (false, self.to_usize())
                }
            }
            Rule::ProgramOwned { program } => {
                msg!("Validating ProgramOwned");
                if let Some(payload_destination) = &payload.destination_key {
                    if let Some(account) = accounts.get(payload_destination) {
                        if *account.owner == *program {
                            return (true, self.to_usize());
                        }
                    }
                }
                (false, self.to_usize())
            }
            Rule::Amount { amount } => {
                msg!("Validating Amount");
                if let Some(payload_amount) = &payload.amount {
                    if amount == payload_amount {
                        (true, self.to_usize())
                    } else {
                        (false, self.to_usize())
                    }
                } else {
                    (false, self.to_usize())
                }
            }
            Rule::Frequency { freq_account } => {
                msg!("Validating Frequency");
                // Deserialize the frequency account
                if let Some(account) = accounts.get(freq_account) {
                    if let Ok(current_time) = solana_program::clock::Clock::get() {
                        let freq_account = FrequencyAccount::from_account_info(account);
                        if let Ok(freq_account) = freq_account {
                            // Grab the current time
                            // Compare  last time + period to current time
                            if let Some(freq_check) =
                                freq_account.last_update.checked_add(freq_account.period)
                            {
                                if freq_check < current_time.unix_timestamp {
                                    (true, self.to_usize())
                                } else {
                                    (false, self.to_usize())
                                }
                            } else {
                                (false, self.to_usize())
                            }
                        } else {
                            (false, self.to_usize())
                        }
                    } else {
                        (false, self.to_usize())
                    }
                } else {
                    (false, self.to_usize())
                }
            }
            Rule::PubkeyTreeMatch { root } => {
                msg!("Validating PubkeyTreeMatch");
                if let Some(LeafInfo { proof, leaf }) = &payload.tree_match_leaf {
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
                        (true, self.to_usize())
                    } else {
                        (false, self.to_usize())
                    }
                } else {
                    (false, self.to_usize())
                }
            }
        }
    }

    pub fn to_usize(&self) -> usize {
        match self {
            Rule::All { .. } => 0,
            Rule::Any { .. } => 1,
            Rule::Not { .. } => 2,
            Rule::AdditionalSigner { .. } => 3,
            Rule::PubkeyMatch { .. } => 4,
            Rule::DerivedKeyMatch { .. } => 5,
            Rule::ProgramOwned { .. } => 6,
            Rule::Amount { .. } => 7,
            Rule::Frequency { .. } => 8,
            Rule::PubkeyTreeMatch { .. } => 9,
        }
    }

    pub fn to_error(rule_type: usize) -> RuleSetError {
        match rule_type {
            0 => todo!(),                                   // Rule::All { .. }
            1 => todo!(),                                   // Rule::Any { .. }
            2 => todo!(),                                   // Rule::Not { .. }
            3 => RuleSetError::AdditionalSignerCheckFailed, // Rule::AdditionalSigner { .. }
            4 => RuleSetError::PubkeyMatchCheckFailed,      // Rule::PubkeyMatch { .. }
            5 => RuleSetError::DerivedKeyMatchCheckFailed,  // Rule::DerivedKeyMatch { .. }
            6 => RuleSetError::ProgramOwnedCheckFailed,     // Rule::ProgramOwned { .. }
            7 => RuleSetError::AmountCheckFailed,           // Rule::Amount { .. }
            8 => RuleSetError::FrequencyCheckFailed,        // Rule::Frequency { .. }
            9 => RuleSetError::PubkeyTreeMatchCheckFailed,  // Rule::PubkeyTreeMatch { .. }
            _ => unreachable!(),
        }
    }
}
