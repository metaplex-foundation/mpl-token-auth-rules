use serde::{Deserialize, Serialize};

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
