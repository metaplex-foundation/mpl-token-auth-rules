use serde::{Deserialize, Serialize};
use solana_program::pubkey::Pubkey;

pub mod additional_signer;

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub enum Validation {
    All { validations: Vec<Validation> },
    Any { validations: Vec<Validation> },
    AdditionalSigner { account: Pubkey },
}
