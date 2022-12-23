//! The definition and associated functions of the `Payload` type that is passed from the program client to the auth rules program for validation.
use crate::error::RuleSetError;
use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use solana_program::{entrypoint::ProgramResult, pubkey::Pubkey};
use std::collections::HashMap;

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
/// A seed path type used by the `DerivedKeyMatch` rule.
pub struct SeedsVec {
    /// The vector of derivation seeds.
    pub seeds: Vec<Vec<u8>>,
}

impl SeedsVec {
    /// Create a new `SeedsVec`.
    pub fn new(seeds: Vec<Vec<u8>>) -> Self {
        Self { seeds }
    }
}

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
/// A proof type used by the `PubkeyTreeMatch` rule.
pub struct LeafInfo {
    /// The leaf the pubkey exists on.
    pub leaf: [u8; 32],
    /// The merkle proof for the leaf.
    pub proof: Vec<[u8; 32]>,
}

impl LeafInfo {
    /// Create a new `LeafInfo`.
    pub fn new(leaf: [u8; 32], proof: Vec<[u8; 32]>) -> Self {
        Self { leaf, proof }
    }
}

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
/// Variants representing the different types represented in a payload.
pub enum PayloadType {
    /// A plain `Pubkey`.
    Pubkey(Pubkey),
    /// Account and Derivation seeds.
    AccountAndSeeds(Pubkey, SeedsVec),
    /// A merkle leaf and proof.
    MerkleProof(LeafInfo),
    /// A plain `u64` used for `Amount`.
    Number(u64),
}

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone, Default)]
/// A wrapper type for the payload hashmap.
pub struct Payload {
    map: HashMap<PayloadKey, PayloadType>,
}

impl Payload {
    /// Create a new empty `Payload`.
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    /// Create a `Payload` from an array of key-value pairs, specified as
    /// `(PayloadKey, PayloadType)` tuples.
    pub fn from<const N: usize>(arr: [(PayloadKey, PayloadType); N]) -> Self {
        Self {
            map: HashMap::from(arr),
        }
    }

    /// Inserts a key-value pair into the `Payload`.  If the `Payload` did not have this key
    ///  present, then `None` is returned.  If the `Payload` did have this key present, the value
    /// is updated, and the old value is returned.  The key is not updated, though; this matters
    /// for types that can be `==` without being identical.  See `std::collections::HashMap`
    /// documentation for more info.
    pub fn insert(&mut self, key: PayloadKey, value: PayloadType) -> Option<PayloadType> {
        self.map.insert(key, value)
    }

    /// Tries to insert a key-value pair into a `Payload`.  If this key is already in the `Payload`
    /// nothing is updated and an error is returned.
    pub fn try_insert(&mut self, key: PayloadKey, value: PayloadType) -> ProgramResult {
        if self.map.get(&key).is_none() {
            self.map.insert(key, value);
            Ok(())
        } else {
            Err(RuleSetError::ValueOccupied.into())
        }
    }

    /// Returns a reference to the value corresponding to the key.
    pub fn get(&self, key: &PayloadKey) -> Option<&PayloadType> {
        self.map.get(key)
    }

    /// Get a reference to the `Pubkey` associated with a key, if and only if the `Payload` value
    /// is the `PayloadType::Pubkey` variant.  Returns `None` if the key is not present in the
    /// `Payload` or the value is a different `PayloadType` variant.
    pub fn get_pubkey(&self, key: &PayloadKey) -> Option<&Pubkey> {
        if let Some(val) = self.map.get(key) {
            match val {
                PayloadType::Pubkey(pubkey) => Some(pubkey),
                _ => None,
            }
        } else {
            None
        }
    }

    /// Get a reference to the `SeedsVec` associated with a key, if and only if the `Payload` value
    /// is the `PayloadType::Seeds` variant.  Returns `None` if the key is not present in the
    /// `Payload` or the value is a different `PayloadType` variant.
    pub fn get_account_and_seeds(&self, key: &PayloadKey) -> Option<(&Pubkey, &SeedsVec)> {
        if let Some(val) = self.map.get(key) {
            match val {
                PayloadType::AccountAndSeeds(account, seeds) => Some((account, seeds)),
                _ => None,
            }
        } else {
            None
        }
    }

    /// Get a reference to the `LeafInfo` associated with a key, if and only if the `Payload` value
    /// is the `PayloadType::MerkleProof` variant.  Returns `None` if the key is not present in the
    /// `Payload` or the value is a different `PayloadType` variant.
    pub fn get_merkle_proof(&self, key: &PayloadKey) -> Option<&LeafInfo> {
        if let Some(val) = self.map.get(key) {
            match val {
                PayloadType::MerkleProof(leaf_info) => Some(leaf_info),
                _ => None,
            }
        } else {
            None
        }
    }

    /// Get the `u64` associated with a key, if and only if the `Payload` value is the
    /// `PayloadType::Number` variant.  Returns `None` if the key is not present in the `Payload`
    /// or the value is a different `PayloadType` variant.
    pub fn get_amount(&self, key: &PayloadKey) -> Option<u64> {
        if let Some(val) = self.map.get(key) {
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

/// An enum representing the different members of a standard token operation.
pub enum PayloadKey {
    /// The target of the operation, e.g. the recipient of a transfer.
    Target,
    /// The holder of the token, e.g. the sender of a transfer.
    Holder,
    /// The authority of a transfer, e.g. the delegate of token.
    Authority,
    /// The amount being transferred.
    Amount,
}
