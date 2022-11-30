pub mod entrypoint;
pub mod error;
pub mod instruction;
pub mod payload;
pub mod pda;
pub mod processor;
pub mod state;
pub mod utils;

pub use payload::{Payload, PayloadVec};
pub use solana_program;

solana_program::declare_id!("auth9SigNpDKz4sJJ1DfCTuZrZNSAgh9sFD3rboVmgg");
