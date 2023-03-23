#[deny(missing_docs)]
pub mod entrypoint;
#[deny(missing_docs)]
pub mod error;
pub mod instruction;
#[deny(missing_docs)]
pub mod payload;
#[deny(missing_docs)]
pub mod pda;
#[deny(missing_docs)]
pub mod processor;
#[deny(missing_docs)]
pub mod state;
pub mod state_v2;
#[deny(missing_docs)]
pub mod utils;

use error::RuleSetError;
pub use solana_program;

/// Max name length for any of the names used in this crate.
pub const MAX_NAME_LENGTH: usize = 32;

/// Versioning for `RuleSet` structs.
pub enum LibVersion {
    V1 = 1,
    V2,
}

impl TryFrom<u8> for LibVersion {
    type Error = RuleSetError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(LibVersion::V1),
            2 => Ok(LibVersion::V2),
            _ => Err(RuleSetError::UnsupportedRuleSetVersion),
        }
    }
}

solana_program::declare_id!("auth9SigNpDKz4sJJ1DfCTuZrZNSAgh9sFD3rboVmgg");
