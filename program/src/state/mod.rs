use serde::{Deserialize, Serialize};

pub mod primitives;
pub mod rules;

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, Hash, PartialOrd)]
pub enum Operation {
    Transfer,
    Delegate,
    SaleTransfer,
    MigrateClass,
}
