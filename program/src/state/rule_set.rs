use crate::{error::RuleSetError, state::Rule};
use serde::{Deserialize, Serialize};
use solana_program::{entrypoint::ProgramResult, pubkey::Pubkey};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, Default)]
pub struct RuleSet {
    /// Name of the RuleSet, used in PDA derivation.
    rule_set_name: String,
    /// Owner (creator) of the RuleSet.
    owner: Pubkey,
    /// A map to determine the `Rule` that belongs to a given `Operation`.
    pub operations: HashMap<u16, Rule>,
}

impl RuleSet {
    /// Create a new empty `RuleSet`.
    pub fn new(rule_set_name: String, owner: Pubkey) -> Self {
        Self {
            rule_set_name,
            owner,
            operations: HashMap::new(),
        }
    }

    /// Get the name of the `RuleSet`.
    pub fn name(&self) -> &str {
        &self.rule_set_name
    }

    /// Get the owner of the `RuleSet`.
    pub fn owner(&self) -> &Pubkey {
        &self.owner
    }

    /// Add a key-value pair into a `RuleSet`.  If this key is already in the `RuleSet`
    /// nothing is updated and an error is returned.
    pub fn add(&mut self, operation: u16, rules: Rule) -> ProgramResult {
        if self.operations.get(&operation).is_none() {
            self.operations.insert(operation, rules);
            Ok(())
        } else {
            Err(RuleSetError::ValueOccupied.into())
        }
    }

    pub fn get(&self, operation: u16) -> Option<&Rule> {
        self.operations.get(&operation)
    }
}
