use std::collections::HashMap;

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{account_info::AccountInfo, pubkey::Pubkey};

use crate::state::Validation;

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub struct AdditionalSigner {
    account: Pubkey,
}

impl AdditionalSigner {
    pub fn new(account: Pubkey) -> Self {
        Self { account }
    }
}

impl Validation for AdditionalSigner {
    fn validate(&self, accounts: &HashMap<Pubkey, &AccountInfo>) -> bool {
        if let Some(account) = accounts.get(&self.account) {
            account.is_signer
        } else {
            false
        }
    }
}
