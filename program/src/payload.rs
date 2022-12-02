use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub struct SeedsVec {
    pub seeds: Vec<Vec<u8>>,
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub struct LeafInfo {
    pub proof: Vec<[u8; 32]>,
    pub leaf: [u8; 32],
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone, Default)]
pub struct Payload {
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
