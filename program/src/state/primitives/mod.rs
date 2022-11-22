use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use solana_program::{account_info::AccountInfo, msg, pubkey::Pubkey};

use crate::data::{AccountTag, Payload};

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub enum Validation {
    All { validations: Vec<Validation> },
    Any { validations: Vec<Validation> },
    AdditionalSigner { account: Pubkey },
    PubkeyMatch { account: Pubkey },
    DerivedKeyMatch { account: Pubkey },
    ProgramOwned { program: Pubkey },
    IdentityAssociated { account: Pubkey },
    Amount { amount: u64 },
    Frequency { freq_account: Pubkey },
    PubkeyTreeMatch { root: [u8; 32] },
}

impl Validation {
    pub fn validate(
        &self,
        accounts: &HashMap<Pubkey, &AccountInfo>,
        tags: &HashMap<AccountTag, Pubkey>,
        payloads: &HashMap<u8, Payload>,
    ) -> bool {
        match self {
            Validation::All { validations } => {
                msg!("Validating All");
                validations
                    .iter()
                    .all(|v| v.validate(accounts, tags, payloads))
            }
            Validation::Any { validations } => {
                msg!("Validating Any");
                validations.iter().any(|v| {
                    msg!("{:#?}", v);
                    v.validate(accounts, tags, payloads)
                })
            }
            Validation::AdditionalSigner { account } => {
                msg!("Validating AdditionalSigner");
                if let Some(account) = accounts.get(account) {
                    account.is_signer
                } else {
                    false
                }
            }
            Validation::PubkeyMatch { account } => {
                msg!("Validating PubkeyMatch");
                tags.get(&AccountTag::Destination).unwrap() == account
            }
            Validation::DerivedKeyMatch { account } => todo!(),
            Validation::ProgramOwned { program } => {
                msg!("Validating ProgramOwned");
                if let Some(account) = accounts.get(program) {
                    account.owner == program
                } else {
                    false
                }
            }
            Validation::IdentityAssociated { account } => todo!(),
            Validation::Amount { amount } => {
                msg!("Validating Amount");
                if let Some(Payload::Amount { amount: a }) = payloads.get(&self.to_u8()) {
                    amount == a
                } else {
                    false
                }
            }
            Validation::Frequency { freq_account } => {
                todo!()
                // Deserialize the frequency account
                // Grab the current time
                // Compare  last time + period to current time
            }
            Validation::PubkeyTreeMatch { root } => {
                msg!("Validating PubkeyTreeMatch");
                if let Some(Payload::PubkeyTreeMatch { proof, leaf }) = payloads.get(&self.to_u8())
                {
                    let mut computed_hash = *leaf;
                    for proof_element in proof.into_iter() {
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
                    computed_hash == *root
                } else {
                    false
                }
            }
        }
    }

    pub fn to_u8(&self) -> u8 {
        match self {
            Validation::All { .. } => 0,
            Validation::Any { .. } => 1,
            Validation::AdditionalSigner { .. } => 2,
            Validation::PubkeyMatch { .. } => 3,
            Validation::DerivedKeyMatch { .. } => 4,
            Validation::ProgramOwned { .. } => 5,
            Validation::IdentityAssociated { .. } => 6,
            Validation::Amount { .. } => 7,
            Validation::Frequency { .. } => 8,
            Validation::PubkeyTreeMatch { .. } => 9,
        }
    }
}
