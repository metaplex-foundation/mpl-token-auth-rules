#[cfg(not(feature = "no-entrypoint"))]
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
pub mod types;
#[deny(missing_docs)]
pub mod utils;

pub use solana_program;

solana_program::declare_id!("auth9SigNpDKz4sJJ1DfCTuZrZNSAgh9sFD3rboVmgg");
