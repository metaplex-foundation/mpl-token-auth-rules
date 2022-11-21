use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::state::{primitives::Validation, Operation};

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, Default)]
pub struct RuleSet {
    operations: HashMap<Operation, Validation>,
}

impl RuleSet {
    pub fn new() -> Self {
        Self {
            operations: HashMap::new(),
        }
    }

    pub fn add(&mut self, operation: Operation, validations: Validation) {
        self.operations.insert(operation, validations);
    }

    pub fn get(&self, operation: Operation) -> Option<&Validation> {
        self.operations.get(&operation)
    }
}
