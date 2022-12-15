use borsh::{BorshDeserialize, BorshSerialize};
#[cfg(feature = "serde-feature")]
use serde::{Deserialize, Serialize};
use solana_program::pubkey::Pubkey;

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
}

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub enum PayloadField {
    Target(PayloadType),
    Holder(PayloadType),
    Authority(PayloadType),
    Amount(u64),
}

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone, Default)]
pub struct ParsedPayload {
    pub target: Option<PayloadType>,
    pub holder: Option<PayloadType>,
    pub authority: Option<PayloadType>,
    pub amount: Option<u64>,
}

impl ParsedPayload {
    pub fn get_pubkey(&self, key: PayloadKey) -> Option<Pubkey> {
        match key {
            PayloadKey::Target => match &self.target {
                Some(PayloadType::Pubkey(pubkey)) => Some(*pubkey),
                _ => None,
            },
            PayloadKey::Holder => match &self.holder {
                Some(PayloadType::Pubkey(pubkey)) => Some(*pubkey),
                _ => None,
            },
            PayloadKey::Authority => match &self.authority {
                Some(PayloadType::Pubkey(pubkey)) => Some(*pubkey),
                _ => None,
            },
            PayloadKey::Amount => None,
        }
    }

    pub fn get_seeds(&self, key: PayloadKey) -> Option<SeedsVec> {
        match key {
            PayloadKey::Target => match &self.target {
                Some(PayloadType::Seeds(seeds)) => Some(seeds.clone()),
                _ => None,
            },
            PayloadKey::Holder => match &self.holder {
                Some(PayloadType::Seeds(seeds)) => Some(seeds.clone()),
                _ => None,
            },
            PayloadKey::Authority => match &self.authority {
                Some(PayloadType::Seeds(seeds)) => Some(seeds.clone()),
                _ => None,
            },
            PayloadKey::Amount => None,
        }
    }

    pub fn get_merkle_proof(&self, key: PayloadKey) -> Option<LeafInfo> {
        match key {
            PayloadKey::Target => match &self.target {
                Some(PayloadType::MerkleProof(leaf_info)) => Some(leaf_info.clone()),
                _ => None,
            },
            PayloadKey::Holder => match &self.holder {
                Some(PayloadType::MerkleProof(leaf_info)) => Some(leaf_info.clone()),
                _ => None,
            },
            PayloadKey::Authority => match &self.authority {
                Some(PayloadType::MerkleProof(leaf_info)) => Some(leaf_info.clone()),
                _ => None,
            },
            PayloadKey::Amount => None,
        }
    }
}

#[repr(C)]
#[derive(
    BorshSerialize, BorshDeserialize, Serialize, Deserialize, PartialEq, Eq, Debug, Clone, Copy,
)]
pub enum PayloadKey {
    Target,
    Holder,
    Authority,
    Amount,
}
