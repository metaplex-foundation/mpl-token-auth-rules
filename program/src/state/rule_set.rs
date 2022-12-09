use crate::state::{Operation, Rule};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, Default)]
pub struct RuleSet {
    pub operations: HashMap<Operation, Rule>,
}

impl RuleSet {
    pub fn new() -> Self {
        Self {
            operations: HashMap::new(),
        }
    }

    pub fn add(&mut self, operation: Operation, rules: Rule) {
        self.operations.insert(operation, rules);
    }

    pub fn get(&self, operation: Operation) -> Option<&Rule> {
        self.operations.get(&operation)
    }
}
