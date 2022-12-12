use crate::{
    error::RuleSetError,
    payload::{LeafInfo, SeedsVec},
    utils::assert_derivation,
    Payload,
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
    ) -> ProgramResult {
        let (status, rollup_err) = self.ll_validate(accounts, payload);

        if status {
            ProgramResult::Ok(())
        } else {
            ProgramResult::Err(rollup_err.into())
        }
    }

    pub fn ll_validate(
        &self,
        accounts: &HashMap<Pubkey, &AccountInfo>,
        payload: &Payload,
    ) -> (bool, RuleSetError) {
        match self {
            Rule::All { rules } => {
                msg!("Validating All");
                let mut last = self.to_error();
                for rule in rules {
                    last = rule.to_error();
                    let result = rule.ll_validate(accounts, payload);
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
                    let result = rule.ll_validate(accounts, payload);
                    if result.0 {
                        return result;
                    }
                }
                (false, last)
            }
            Rule::Not { rule } => {
                let result = rule.ll_validate(accounts, payload);
                (!result.0, result.1)
            }
            Rule::AdditionalSigner { account } => {
                msg!("Validating AdditionalSigner");
                if let Some(signer) = accounts.get(account) {
                    (signer.is_signer, self.to_error())
                } else {
                    (false, self.to_error())
                }
            }
            Rule::PubkeyMatch { destination } => {
                msg!("Validating PubkeyMatch");
                if let Some(payload_destination) = &payload.destination_key {
                    if destination == payload_destination {
                        (true, self.to_error())
                    } else {
                        (false, self.to_error())
                    }
                } else {
                    (false, self.to_error())
                }
            }
            Rule::DerivedKeyMatch { account } => {
                msg!("Validating DerivedKeyMatch");
                if let Some(SeedsVec { seeds }) = &payload.derived_key_seeds {
                    if let Some(account) = accounts.get(account) {
                        let vec_of_slices = seeds.iter().map(Vec::as_slice).collect::<Vec<&[u8]>>();
                        let seeds = &vec_of_slices[..];
                        if let Ok(_bump) = assert_derivation(&crate::id(), account, seeds) {
                            (true, self.to_error())
                        } else {
                            (false, self.to_error())
                        }
                    } else {
                        (false, self.to_error())
                    }
                } else {
                    (false, self.to_error())
                }
            }
            Rule::ProgramOwned { program } => {
                msg!("Validating ProgramOwned");
                if let Some(payload_destination) = &payload.destination_key {
                    if let Some(account) = accounts.get(payload_destination) {
                        if *account.owner == *program {
                            return (true, self.to_error());
                        }
                    }
                }
                (false, self.to_error())
            }
            Rule::Amount { amount } => {
                msg!("Validating Amount");
                if let Some(payload_amount) = &payload.amount {
                    if amount == payload_amount {
                        (true, self.to_error())
                    } else {
                        (false, self.to_error())
                    }
                } else {
                    (false, self.to_error())
                }
            }
            #[allow(unused_variables)]
            Rule::Frequency { freq_account } => {
                msg!("Validating Frequency");
                // TODO Rule is not implemented.
                return (false, RuleSetError::NotImplemented);
                #[allow(unreachable_code)]
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
                                    (true, self.to_error())
                                } else {
                                    (false, self.to_error())
                                }
                            } else {
                                (false, self.to_error())
                            }
                        } else {
                            (false, self.to_error())
                        }
                    } else {
                        (false, self.to_error())
                    }
                } else {
                    (false, self.to_error())
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
                        (true, self.to_error())
                    } else {
                        (false, self.to_error())
                    }
                } else {
                    (false, self.to_error())
                }
            }
        }
    }

    pub fn to_error(&self) -> RuleSetError {
        match self {
            Rule::AdditionalSigner { .. } => RuleSetError::AdditionalSignerCheckFailed,
            Rule::PubkeyMatch { .. } => RuleSetError::PubkeyMatchCheckFailed,
            Rule::DerivedKeyMatch { .. } => RuleSetError::DerivedKeyMatchCheckFailed,
            Rule::ProgramOwned { .. } => RuleSetError::ProgramOwnedCheckFailed,
            Rule::Amount { .. } => RuleSetError::AmountCheckFailed,
            Rule::Frequency { .. } => RuleSetError::FrequencyCheckFailed,
            Rule::PubkeyTreeMatch { .. } => RuleSetError::PubkeyTreeMatchCheckFailed,
            _ => RuleSetError::NotImplemented,
        }
    }
}
