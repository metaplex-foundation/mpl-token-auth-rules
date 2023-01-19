//! `RuleSet` types
//!
//! This file contains the the main types used to store `RuleSet` data in the `RuleSet` PDA on
//! chain.  It includes the main `RuleSet` types which keeps the the map of operations to `Rules`,
//! as well as header and revision map types used to manage data within the `RuleSet` PDA.
//!
//! Each time a `RuleSet` is updated, a new revision is added to the PDA, and previous revisions
//! never deleted.  The revision map is needed so that during `RuleSet` validation the desired
//! revision can be selected by the user.
//!
//! Because the `RuleSet`s and the revision map are variable size, a fixed size header is stored
//! at the beginning of the `RuleSet` PDA that allows new `RuleSets` and updated revision maps
//! to be added to the PDA without moving the previous revision `RuleSets` and without losing the
//! revision map's location.
//!
//! Also note there is a 1-byte version preceding each `RuleSet` revision and the revision map.
//! This is not included in the data struct itself to give flexibility to update `RuleSet`s and
//! the revision map data structs and even change serialization format.
//!
//! RuleSet PDA data layout
//! | Header  | RuleSet | RuleSet revision 0 | RuleSet | RuleSet revision 1 | ... | Rev map | RuleSetRevisionMap |
//! |         | version |                    | version |                    |     | version |                    |
//! |---------|---------|--------------------|---------|--------------------|-----|---------|--------------------|
//! | 8 bytes | 1 byte  | variable bytes     | 1 byte  | variable bytes     | ... | 1 byte  | variable bytes     |

use crate::{error::RuleSetError, state::Rule};
use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use solana_program::{entrypoint::ProgramResult, pubkey::Pubkey};
use std::collections::HashMap;

/// Version of the `RuleSetRevisionMapV1` struct.
pub const RULE_SET_REV_MAP_VERSION: u8 = 1;

/// Version of the `RuleSetV1` struct.
pub const RULE_SET_LIB_VERSION: u8 = 1;

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
/// Header used to keep track of where RuleSets are stored in the PDA.  This header is meant
/// to be stored at the beginning of the PDA and never be versioned so that it always
/// has the same serialized size.
pub struct RuleSetHeader {
    /// The location of revision map version stored in the PDA.  This is one byte before the
    /// revision map itself.
    pub rev_map_version_location: usize,
}

impl RuleSetHeader {
    /// Create a new `RuleSetHeader`.
    pub fn new(rev_map_version_location: usize) -> Self {
        Self {
            rev_map_version_location,
        }
    }
}

/// Size of `RuleSetHeader` when Borsh serialized.
pub const RULE_SET_SERIALIZED_HEADER_LEN: usize = 8;

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone, Default)]
/// Revision map used to keep track of where individual `RuleSet` revisions are stored in the PDA.
pub struct RuleSetRevisionMapV1 {
    /// `Vec` used to map a `RuleSet` revision number to its location in the PDA.
    pub rule_set_revisions: Vec<usize>,
    /// The current maximum revision stored in the PDA (essentially the greatest
    /// element of `rule_set_revisions`).
    pub max_revision: usize,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
/// The struct containing all Rule Set data, most importantly the map of operations to `Rules`.
pub struct RuleSetV1 {
    /// Version of the RuleSet.  This is not a user version, but the version
    /// of this lib, to make sure that a `RuleSet` passed into our handlers
    /// is one we are compatible with.
    lib_version: u8,
    /// Name of the RuleSet, used in PDA derivation.
    rule_set_name: String,
    /// Owner (creator) of the RuleSet.
    owner: Pubkey,
    /// A map to determine the `Rule` that belongs to a given `Operation`.
    pub operations: HashMap<String, Rule>,
}

impl RuleSetV1 {
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
    pub fn lib_version(&self) -> u8 {
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
