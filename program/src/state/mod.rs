use std::{collections::HashMap, fmt};

use serde::{Deserialize, Serialize};
use solana_program::{account_info::AccountInfo, pubkey::Pubkey};

pub mod primitives;
pub mod rules;

// pub trait Validation:
//     Serialize + Deserialize + PartialEq + Eq + fmt::Debug + Clone
// {
//     fn validate(&self, accounts: &HashMap<Pubkey, &AccountInfo>) -> bool;
// }

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, Hash, PartialOrd)]
pub enum Operation {
    Transfer,
    Delegate,
    SaleTransfer,
    MigrateClass,
}
