use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use solana_program::{account_info::AccountInfo, pubkey::Pubkey};

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub enum Validation {
    All { validations: Vec<Validation> },
    Any { validations: Vec<Validation> },
    AdditionalSigner { account: Pubkey },
}

impl Validation {
    pub fn validate(&self, accounts: &HashMap<Pubkey, &AccountInfo>) -> bool {
        match self {
            Validation::All { validations } => validations.iter().all(|v| v.validate(accounts)),
            Validation::Any { validations } => validations.iter().any(|v| v.validate(accounts)),
            Validation::AdditionalSigner { account } => {
                if let Some(account) = accounts.get(account) {
                    account.is_signer
                } else {
                    false
                }
            }
        }
    }
}
