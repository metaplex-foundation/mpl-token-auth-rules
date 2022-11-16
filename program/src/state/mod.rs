use std::{collections::HashMap, fmt};

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{account_info::AccountInfo, pubkey::Pubkey};

pub mod primitives;
pub mod rules;

pub trait Validation:
    BorshSerialize + BorshDeserialize + PartialEq + Eq + fmt::Debug + Clone
{
    fn validate(&self, accounts: &HashMap<Pubkey, &AccountInfo>) -> bool;
}

#[derive(Eq, Hash, PartialEq, Debug)]
pub enum Operation {
    Transfer,
    Delegate,
    SaleTransfer,
    MigrateClass,
}
