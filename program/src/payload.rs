//! The definition and associated functions of the `Payload` type that is passed from the program client to the auth rules program for validation.
use crate::error::RuleSetError;
use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use solana_program::{entrypoint::ProgramResult, pubkey::Pubkey};
use std::collections::HashMap;

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
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
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
/// A proof type used by the `PubkeyTreeMatch` rule.
pub struct ProofInfo {
    /// The merkle proof.
    pub proof: Vec<[u8; 32]>,
}

impl ProofInfo {
    /// Create a new `ProofInfo`.
    pub fn new(proof: Vec<[u8; 32]>) -> Self {
        Self { proof }
    }
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
/// Variants representing the different types represented in a payload.
pub enum PayloadType {
    /// A plain `Pubkey`.
    Pubkey(Pubkey),
    /// PDA derivation seeds.
    Seeds(SeedsVec),
    /// A merkle proof.
    MerkleProof(ProofInfo),
    /// A plain `u64` used for `Amount`.
    Number(u64),
}

#[repr(C)]
#[derive(
    BorshSerialize, BorshDeserialize, Serialize, Deserialize, PartialEq, Eq, Debug, Clone, Default,
)]
/// A wrapper type for the payload hashmap.
pub struct Payload {
    map: HashMap<String, PayloadType>,
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
    pub fn from<const N: usize>(arr: [(String, PayloadType); N]) -> Self {
        Self {
            map: HashMap::from(arr),
        }
    }

    /// Inserts a key-value pair into the `Payload`.  If the `Payload` did not have this key
    ///  present, then `None` is returned.  If the `Payload` did have this key present, the value
    /// is updated, and the old value is returned.  The key is not updated, though; this matters
    /// for types that can be `==` without being identical.  See `std::collections::HashMap`
    /// documentation for more info.
    pub fn insert(&mut self, key: String, value: PayloadType) -> Option<PayloadType> {
        self.map.insert(key, value)
    }

    /// Tries to insert a key-value pair into a `Payload`.  If this key is already in the `Payload`
    /// nothing is updated and an error is returned.
    pub fn try_insert(&mut self, key: String, value: PayloadType) -> ProgramResult {
        if self.map.get(&key).is_none() {
            self.map.insert(key, value);
            Ok(())
        } else {
            Err(RuleSetError::ValueOccupied.into())
        }
    }

    /// Returns a reference to the value corresponding to the key.
    pub fn get(&self, key: &String) -> Option<&PayloadType> {
        self.map.get(key)
    }

    /// Get a reference to the `Pubkey` associated with a key, if and only if the `Payload` value
    /// is the `PayloadType::Pubkey` variant.  Returns `None` if the key is not present in the
    /// `Payload` or the value is a different `PayloadType` variant.
    pub fn get_pubkey(&self, key: &String) -> Option<&Pubkey> {
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
    pub fn get_seeds(&self, key: &String) -> Option<&SeedsVec> {
        if let Some(val) = self.map.get(key) {
            match val {
                PayloadType::Seeds(seeds) => Some(seeds),
                _ => None,
            }
        } else {
            None
        }
    }

    /// Get a reference to the `ProofInfo` associated with a key, if and only if the `Payload` value
    /// is the `PayloadType::MerkleProof` variant.  Returns `None` if the key is not present in the
    /// `Payload` or the value is a different `PayloadType` variant.
    pub fn get_merkle_proof(&self, key: &String) -> Option<&ProofInfo> {
        if let Some(val) = self.map.get(key) {
            match val {
                PayloadType::MerkleProof(proof_info) => Some(proof_info),
                _ => None,
            }
        } else {
            None
        }
    }

    /// Get the `u64` associated with a key, if and only if the `Payload` value is the
    /// `PayloadType::Number` variant.  Returns `None` if the key is not present in the `Payload`
    /// or the value is a different `PayloadType` variant.
    pub fn get_amount(&self, key: &String) -> Option<u64> {
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
