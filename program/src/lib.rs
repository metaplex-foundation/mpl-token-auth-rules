pub mod entrypoint;
pub mod error;
pub mod instruction;
pub mod payload;
pub mod pda;
pub mod processor;
pub mod state;
#[deny(missing_docs)]
pub mod utils;

pub use solana_program;

/// Max name length for any of the names used in this crate.
pub const MAX_NAME_LENGTH: usize = 32;

solana_program::declare_id!("auth9SigNpDKz4sJJ1DfCTuZrZNSAgh9sFD3rboVmgg");
