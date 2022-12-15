use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use solana_program::pubkey::Pubkey;
use std::collections::HashMap;

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub struct SeedsVec {
    pub seeds: Vec<Vec<u8>>,
}

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub struct LeafInfo {
    pub leaf: [u8; 32],
    pub proof: Vec<[u8; 32]>,
}

impl LeafInfo {
    pub fn new(leaf: [u8; 32], proof: Vec<[u8; 32]>) -> Self {
        Self { leaf, proof }
    }
}

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub enum PayloadType {
    Pubkey(Pubkey),
    Seeds(SeedsVec),
    MerkleProof(LeafInfo),
    Number(u64),
}

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone, Default)]
pub struct Payload(HashMap<PayloadKey, PayloadType>);

impl Payload {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn from(map: HashMap<PayloadKey, PayloadType>) -> Self {
        Self(map)
    }

    pub fn get_pubkey(&self, key: &PayloadKey) -> Option<Pubkey> {
        if let Some(val) = self.0.get(key) {
            match val {
                PayloadType::Pubkey(pubkey) => Some(*pubkey),
                _ => None,
            }
        } else {
            None
        }
    }

    pub fn get_seeds(&self, key: &PayloadKey) -> Option<SeedsVec> {
        if let Some(val) = self.0.get(key) {
            match val {
                PayloadType::Seeds(seeds) => Some(seeds.clone()),
                _ => None,
            }
        } else {
            None
        }
    }

    pub fn get_merkle_proof(&self, key: &PayloadKey) -> Option<LeafInfo> {
        if let Some(val) = self.0.get(key) {
            match val {
                PayloadType::MerkleProof(leaf_info) => Some(leaf_info.clone()),
                _ => None,
            }
        } else {
            None
        }
    }

    pub fn get_amount(&self, key: &PayloadKey) -> Option<u64> {
        if let Some(val) = self.0.get(key) {
            match val {
                PayloadType::Number(number) => Some(*number),
                _ => None,
            }
        } else {
            None
        }
    }
}

#[repr(C)]
#[derive(
    BorshSerialize,
    BorshDeserialize,
    Serialize,
    Deserialize,
    PartialOrd,
    Hash,
    PartialEq,
    Eq,
    Debug,
    Clone,
    Copy,
)]
pub enum PayloadKey {
    Target,
    Holder,
    Authority,
    Amount,
}
