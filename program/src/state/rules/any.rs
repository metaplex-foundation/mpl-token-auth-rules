use std::collections::HashMap;

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{account_info::AccountInfo, pubkey::Pubkey};

use crate::state::Validation;

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub struct Any<'a> {
    validations: Vec<Box<dyn Validation + 'a>>,
}

impl<'a> Any<'a> {
    pub fn new(validations: Vec<Box<dyn Validation + 'a>>) -> Self {
        Self { validations }
    }
}

impl<'a> Validation for Any<'a> {
    fn validate(&self, accounts: &HashMap<Pubkey, &AccountInfo>) -> bool {
        self.validations.iter().any(|v| v.validate(accounts))
    }
}
