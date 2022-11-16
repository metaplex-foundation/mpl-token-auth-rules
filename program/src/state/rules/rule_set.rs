use std::collections::HashMap;

use borsh::{BorshDeserialize, BorshSerialize};

use crate::state::{Operation, Validation};

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub struct RuleSet<'a> {
    operations: HashMap<Operation, Box<dyn Validation + 'a>>,
}

impl<'a> RuleSet<'a> {
    pub fn new() -> Self {
        Self {
            operations: HashMap::new(),
        }
    }

    pub fn add(&mut self, operation: Operation, validations: Box<dyn Validation + 'a>) {
        self.operations.insert(operation, validations);
    }

    pub fn get(&self, operation: Operation) -> Option<&dyn Validation> {
        self.operations.get(&operation).map(|v| &**v)
    }
}
