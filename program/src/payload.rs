use serde::{Deserialize, Serialize};
use solana_program::pubkey::Pubkey;

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Payload<'a> {
    All,
    Any,
    AdditionalSigner,
    PubkeyMatch {
        destination: Pubkey,
    },
    DerivedKeyMatch {
        seeds: &'a Vec<&'a [u8]>,
    },
    ProgramOwned,
    Amount {
        amount: u64,
    },
    Frequency,
    PubkeyTreeMatch {
        proof: Vec<[u8; 32]>,
        leaf: [u8; 32],
    },
}
