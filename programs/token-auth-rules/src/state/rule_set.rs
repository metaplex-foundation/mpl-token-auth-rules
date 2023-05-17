/// See state module for description of PDA memory layout.
use crate::{
    error::RuleSetError,
    state::{Key, Rule},
    types::{Assertable, LibVersion, RuleSet},
};
use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
#[cfg(feature = "serde-with-feature")]
use serde_with::{As, DisplayFromStr};
use solana_program::{entrypoint::ProgramResult, program_error::ProgramError, pubkey::Pubkey};
use std::collections::HashMap;

/// Version of the `RuleSetRevisionMapV1` struct.
pub const RULE_SET_REV_MAP_VERSION: u8 = 1;

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
/// Header used to keep track of where RuleSets are stored in the PDA.  This header is meant
/// to be stored at the beginning of the PDA and never be versioned so that it always
/// has the same serialized size.  See top-level module for description of PDA memory layout.
pub struct RuleSetHeader {
    /// The `Key` for this account which identifies it as a `RuleSet` account.
    pub key: Key,
    /// The location of revision map version stored in the PDA.  This is one byte before the
    /// revision map itself.
    pub rev_map_version_location: usize,
}

impl RuleSetHeader {
    /// Create a new `RuleSetHeader`.
    pub fn new(rev_map_version_location: usize) -> Self {
        Self {
            key: Key::RuleSet,
            rev_map_version_location,
        }
    }
}

/// Size of `RuleSetHeader` when Borsh serialized.
pub const RULE_SET_SERIALIZED_HEADER_LEN: usize = 9;

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone, Default)]
/// Revision map used to keep track of where individual `RuleSet` revisions are stored in the PDA.
/// See top-level module for description of PDA memory layout.
pub struct RuleSetRevisionMapV1 {
    /// `Vec` used to map a `RuleSet` revision number to its location in the PDA.
    pub rule_set_revisions: Vec<usize>,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
/// The struct containing all Rule Set data, most importantly the map of operations to `Rules`.
///  See top-level module for description of PDA memory layout.
pub struct RuleSetV1 {
    /// Version of the RuleSet.  This is not a user version, but the version
    /// of this lib, to make sure that a `RuleSet` passed into our handlers
    /// is one we are compatible with.
    lib_version: u8,
    /// Owner (creator) of the RuleSet.
    #[cfg_attr(feature = "serde-with-feature", serde(with = "As::<DisplayFromStr>"))]
    owner: Pubkey,
    /// Name of the RuleSet, used in PDA derivation.
    rule_set_name: String,
    /// A map to determine the `Rule` that belongs to a given `Operation`.
    pub operations: HashMap<String, Rule>,
}

impl RuleSetV1 {
    /// Create a new empty `RuleSet`.
    pub fn new(rule_set_name: String, owner: Pubkey) -> Self {
        Self {
            lib_version: LibVersion::V1 as u8,
            rule_set_name,
            owner,
            operations: HashMap::new(),
        }
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

impl<'a> RuleSet<'a> for RuleSetV1 {
    /// Get the name of the `RuleSet`.
    fn name(&self) -> String {
        self.rule_set_name.clone()
    }

    fn owner(&self) -> &Pubkey {
        &self.owner
    }

    fn lib_version(&self) -> u8 {
        self.lib_version
    }

    /// This function returns the rule for an operation by recursively searching through fallbacks
    fn get_rule(&self, operation: String) -> Result<&dyn Assertable<'a>, ProgramError> {
        let rule = self.get(operation.to_string());

        match rule {
            Some(Rule::Namespace) => {
                // Check for a ':' namespace separator. If it exists try to operation namespace to see if
                // a fallback exists. E.g. 'transfer:owner' will check for a fallback for 'transfer'.
                // If it doesn't exist then fail.
                let split = operation.split(':').collect::<Vec<&str>>();
                if split.len() > 1 {
                    self.get_rule(split[0].to_owned())
                } else {
                    Err(RuleSetError::OperationNotFound.into())
                }
            }
            Some(r) => Ok(r),
            None => Err(RuleSetError::OperationNotFound.into()),
        }
    }
}
