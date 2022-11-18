use std::collections::HashMap;

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{account_info::AccountInfo, pubkey::Pubkey};

use crate::state::primitives::Validation;

// #[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
// pub struct All {
//     validations: Vec<Validation>,
// }

// impl<'a> All {
//     pub fn new(validations: Vec<Validation>) -> Self {
//         Self { validations }
//     }
// }

// impl<'a> Validation for All<'a> {
//     fn validate(&self, accounts: &HashMap<Pubkey, &AccountInfo>) -> bool {
//         self.validations.iter().all(|v| v.validate(accounts))
//     }
// }
