use borsh::{BorshDeserialize, BorshSerialize};
use shank::ShankAccount;

use super::{Key, SolanaAccount};

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone, ShankAccount)]
/// An account containing frequency state.
pub struct FrequencyAccount {
    /// The `Key` for this account which identifies it as a Frequency account.
    pub key: Key,
    /// The last time the frequency counter was updated.
    pub last_update: i64,
    /// The period which must transpire before the rule will succeed again.
    pub period: i64,
}

impl FrequencyAccount {
    /// Create a new `FrequencyAccount`.
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
}
