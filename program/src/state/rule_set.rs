use crate::{error::RuleSetError, state::Rule};
use serde::{Deserialize, Serialize};
use solana_program::{entrypoint::ProgramResult, pubkey::Pubkey};
use std::collections::HashMap;

pub const RULE_SET_VERSION: u32 = 1;

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct RuleSet {
    /// Version of the RuleSet.  This is not a user version, but the version
    /// of this lib, to be used for future backwards compatibility.
    version: u32,
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
            version: RULE_SET_VERSION,
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
    pub fn version(&self) -> u32 {
        self.version
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

    pub fn get(&self, operation: String) -> Option<&Rule> {
        self.operations.get(&operation)
    }
}
