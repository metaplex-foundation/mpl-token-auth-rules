use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use solana_program::{account_info::AccountInfo, blake3::Hash, pubkey::Pubkey};

use crate::data::AccountTag;

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
    PubkeyTreeMatch { root: Pubkey },
}

impl Validation {
    pub fn validate(
        &self,
        accounts: &HashMap<Pubkey, &AccountInfo>,
        tags: &HashMap<AccountTag, Pubkey>,
    ) -> bool {
        match self {
            Validation::All { validations } => {
                validations.iter().all(|v| v.validate(accounts, tags))
            }
            Validation::Any { validations } => {
                validations.iter().any(|v| v.validate(accounts, tags))
            }
            Validation::AdditionalSigner { account } => {
                if let Some(account) = accounts.get(account) {
                    account.is_signer
                } else {
                    false
                }
            }
            Validation::PubkeyMatch { account } => {
                tags.get(&AccountTag::Destination).unwrap() == account
            }
            Validation::DerivedKeyMatch { account } => todo!(),
            Validation::ProgramOwned { program } => {
                if let Some(account) = accounts.get(program) {
                    account.owner == program
                } else {
                    false
                }
            }
            Validation::IdentityAssociated { account } => todo!(),
            Validation::Amount { amount } => todo!(),
            Validation::Frequency { freq_account } => todo!(),
            Validation::PubkeyTreeMatch { root } => todo!(),
        }
    }
}
