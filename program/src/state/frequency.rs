use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::clock::UnixTimestamp;

use super::{Key, SolanaAccount};

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub struct FrequencyAccount {
    pub key: Key,
    pub last_update: UnixTimestamp,
    pub period: UnixTimestamp,
}

impl SolanaAccount for FrequencyAccount {
    fn key() -> Key {
        Key::Frequency
    }

    fn size() -> usize {
        0
    }
}
