use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;
#[cfg(feature = "serde-feature")]
use {
    serde::{Deserialize, Serialize},
    serde_with::{As, DisplayFromStr},
};

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
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone, Default)]
pub struct Payload {
    #[cfg_attr(
        feature = "serde-feature",
        serde(with = "As::<Option<DisplayFromStr>>")
    )]
    pub destination_key: Option<Pubkey>,
    pub derived_key_seeds: Option<SeedsVec>,
    pub amount: Option<u64>,
    pub tree_match_leaf: Option<LeafInfo>,
}

impl Payload {
    pub fn new(
        destination_key: Option<Pubkey>,
        derived_key_seeds: Option<SeedsVec>,
        amount: Option<u64>,
        tree_match_leaf: Option<LeafInfo>,
    ) -> Self {
        Self {
            destination_key,
            derived_key_seeds,
            amount,
            tree_match_leaf,
        }
    }
}
