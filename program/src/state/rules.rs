use crate::data::{AccountTag, Payload};
use serde::{Deserialize, Serialize};
use solana_program::{account_info::AccountInfo, msg, pubkey::Pubkey};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub enum Rule {
    All { rules: Vec<Rule> },
    Any { rules: Vec<Rule> },
    AdditionalSigner { account: Pubkey },
    PubkeyMatch { account: Pubkey },
    DerivedKeyMatch { account: Pubkey },
    ProgramOwned { program: Pubkey },
    IdentityAssociated { account: Pubkey },
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
    ) -> bool {
        match self {
            Rule::All { rules } => {
                msg!("Validating All");
                rules.iter().all(|v| v.validate(accounts, tags, payloads))
            }
            Rule::Any { rules } => {
                msg!("Validating Any");
                rules.iter().any(|v| {
                    msg!("{:#?}", v);
                    v.validate(accounts, tags, payloads)
                })
            }
            Rule::AdditionalSigner { account } => {
                msg!("Validating AdditionalSigner");
                if let Some(account) = accounts.get(account) {
                    account.is_signer
                } else {
                    false
                }
            }
            Rule::PubkeyMatch { account } => {
                msg!("Validating PubkeyMatch");
                tags.get(&AccountTag::Destination).unwrap() == account
            }
            Rule::DerivedKeyMatch { account } => todo!(),
            Rule::ProgramOwned { program } => {
                msg!("Validating ProgramOwned");
                if let Some(account) = accounts.get(program) {
                    account.owner == program
                } else {
                    false
                }
            }
            Rule::IdentityAssociated { account } => todo!(),
            Rule::Amount { amount } => {
                msg!("Validating Amount");
                if let Some(Payload::Amount { amount: a }) = payloads.get(&self.to_u8()) {
                    amount == a
                } else {
                    false
                }
            }
            Rule::Frequency { freq_account } => {
                todo!()
                // Deserialize the frequency account
                // Grab the current time
                // Compare  last time + period to current time
            }
            Rule::PubkeyTreeMatch { root } => {
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
            Rule::All { .. } => 0,
            Rule::Any { .. } => 1,
            Rule::AdditionalSigner { .. } => 2,
            Rule::PubkeyMatch { .. } => 3,
            Rule::DerivedKeyMatch { .. } => 4,
            Rule::ProgramOwned { .. } => 5,
            Rule::IdentityAssociated { .. } => 6,
            Rule::Amount { .. } => 7,
            Rule::Frequency { .. } => 8,
            Rule::PubkeyTreeMatch { .. } => 9,
        }
    }
}
