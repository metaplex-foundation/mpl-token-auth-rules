use serde::{Deserialize, Serialize};

mod rule_set;
mod rules;

pub use rule_set::*;
pub use rules::*;

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, Hash, PartialOrd)]
pub enum Operation {
    Transfer,
    Delegate,
    SaleTransfer,
    MigrateClass,
}
