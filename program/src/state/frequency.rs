use borsh::{BorshDeserialize, BorshSerialize};
use shank::ShankAccount;

use super::{Key, SolanaAccount};

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone, ShankAccount)]
pub struct FrequencyAccount {
    pub key: Key,
    pub last_update: i64,
    pub period: i64,
}

impl FrequencyAccount {
    pub fn new(last_update: i64, period: i64) -> Self {
        Self {
            key: Key::Frequency,
            last_update,
            period,
        }
    }
}

impl SolanaAccount for FrequencyAccount {
    fn key() -> Key {
        Key::Frequency
    }

    fn size() -> usize {
        1   // key
        + 8 // last_update
        + 8 // period
    }
}
