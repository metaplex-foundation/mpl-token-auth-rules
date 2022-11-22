use serde::{Deserialize, Serialize};
use solana_program::clock::UnixTimestamp;

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, Hash)]
pub enum AccountTag {
    Source,
    Destination,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Payload<'a> {
    All,
    Any,
    AdditionalSigner,
    PubkeyMatch,
    DerivedKeyMatch {
        keys: &'a Vec<&'a [u8]>,
    },
    ProgramOwned,
    IdentityAssociated,
    Amount {
        amount: u64,
    },
    Frequency,
    PubkeyTreeMatch {
        proof: Vec<[u8; 32]>,
        leaf: [u8; 32],
    },
}
