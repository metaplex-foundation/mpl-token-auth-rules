use crate::{error::RuleSetError, state::Rule};
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;
use solana_program::{entrypoint::ProgramResult, pubkey::Pubkey};
use std::collections::HashMap;

/// Version of the `RuleSetHeader` struct.
pub const RULE_SET_LIB_HEADER_VERSION: u8 = 1;

/// Version of the `RuleSet` struct.
pub const RULE_SET_LIB_VERSION: u32 = 1;

/// Max number of `RuleSet`s that can be saved.
pub const MAX_RULE_SETS: usize = 64;

/// Size of `RuleSetHeader` when Borsh serialized.
pub const RULE_SET_SERIALIZED_HEADER_LEN: usize = 521;

use borsh::{BorshDeserialize, BorshSerialize};

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
#[serde(rename_all = "camelCase")]
/// Header used to keep track of where RuleSets are stored in the PDA.
pub struct RuleSetHeader {
    /// Version of the RuleSetHeader.  This is not a user version, but the version
    /// of this lib, to be used for future backwards compatibility.
    lib_version: u8,
    /// Array used to map a RuleSet version number to its offset in the PDA.
    #[serde(with = "BigArray")]
    pub rule_set_locs: [usize; MAX_RULE_SETS],
    /// The current maximum version stored in the PDA.
    pub max_version: usize,
}

impl Default for RuleSetHeader {
    fn default() -> Self {
        Self {
            lib_version: 1,
            rule_set_locs: [0; MAX_RULE_SETS],
            max_version: 0,
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
/// The struct containing all Rule Set data.
pub struct RuleSet {
    /// Version of the RuleSet.  This is not a user version, but the version
    /// of this lib, to be used for future backwards compatibility.
    lib_version: u32,
    /// Name of the RuleSet, used in PDA derivation.
    rule_set_name: String,
    /// Owner (creator) of the RuleSet.
    owner: Pubkey,
    /// A map to determine the `Rule` that belongs to a given `Operation`.
    pub operations: HashMap<String, Rule>,
}

impl RuleSet {
    /// Create a new empty `RuleSet`.
    pub fn new(rule_set_name: String, owner: Pubkey) -> Self {
        Self {
            lib_version: RULE_SET_LIB_VERSION,
            rule_set_name,
            owner,
            operations: HashMap::new(),
        }
    }

    /// Get the name of the `RuleSet`.
    pub fn name(&self) -> &str {
        &self.rule_set_name
    }

    /// Get the version of the `RuleSet`.
    pub fn lib_version(&self) -> u32 {
        self.lib_version
    }

    /// Get the owner of the `RuleSet`.
    pub fn owner(&self) -> &Pubkey {
        &self.owner
    }

    /// Add a key-value pair into a `RuleSet`.  If this key is already in the `RuleSet`
    /// nothing is updated and an error is returned.
    pub fn add(&mut self, operation: String, rules: Rule) -> ProgramResult {
        if self.operations.get(&operation).is_none() {
            self.operations.insert(operation, rules);
            Ok(())
        } else {
            Err(RuleSetError::ValueOccupied.into())
        }
    }

    /// Retrieve the `Rule` tree for a given `Operation`.
    pub fn get(&self, operation: String) -> Option<&Rule> {
        self.operations.get(&operation)
    }
}
