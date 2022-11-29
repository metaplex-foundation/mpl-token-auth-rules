use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub enum Payload {
    All,
    Any,
    AdditionalSigner,
    PubkeyMatch {
        destination: Pubkey,
    },
    DerivedKeyMatch {
        seeds: Vec<Vec<u8>>,
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
