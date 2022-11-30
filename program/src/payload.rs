use crate::{error::RuleSetError, state::Rule};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

/// Number of variants in the Payload enum.  Must be kept up to date
/// until `std::mem::variant_count` is stabilized.
const NUM_PAYLOAD_VARIANTS: usize = 10;

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub enum Payload {
    None,
    All,
    Any,
    AdditionalSigner,
    PubkeyMatch {
        destination: Pubkey,
    },
    DerivedKeyMatch {
        seeds: Vec<Vec<u8>>,
    },
    ProgramOwned,
    Amount {
        amount: u64,
    },
    Frequency,
    PubkeyTreeMatch {
        proof: Vec<[u8; 32]>,
        leaf: [u8; 32],
    },
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub struct PayloadVec {
    payloads: Vec<Payload>,
}

impl PayloadVec {
    pub fn new() -> Self {
        Self {
            payloads: vec![Payload::None; NUM_PAYLOAD_VARIANTS],
        }
    }

    pub fn add(&mut self, rule: &Rule, payload: Payload) -> Result<(), RuleSetError> {
        let index = rule.to_usize();
        if index >= NUM_PAYLOAD_VARIANTS {
            return Err(RuleSetError::PayloadVecIndexError);
        }
        self.payloads[index] = payload;
        Ok(())
    }

    pub fn get(&self, rule: &Rule) -> Option<&Payload> {
        match self.payloads.get(rule.to_usize()) {
            Some(Payload::None) => None,
            other => other,
        }
    }
}

impl Default for PayloadVec {
    fn default() -> Self {
        Self::new()
    }
}
