/// See state module for description of PDA memory layout.
use crate::{
    error::RuleSetError,
    state::{Key, Rule},
};
use borsh::{BorshDeserialize, BorshSerialize};
use serde::{
    de::{DeserializeSeed, Deserializer, IgnoredAny, MapAccess, SeqAccess, Visitor},
    Deserialize, Serialize,
};
#[cfg(feature = "serde-with-feature")]
use serde_with::{As, DisplayFromStr};
use solana_program::{entrypoint::ProgramResult, log::sol_log_compute_units, pubkey::Pubkey};
use std::{collections::HashMap, marker::PhantomData};

/// Version of the `RuleSetRevisionMapV1` struct.
pub const RULE_SET_REV_MAP_VERSION: u8 = 1;

/// Version of the `RuleSetV1` struct.
pub const RULE_SET_LIB_VERSION: u8 = 1;

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
/// Header used to keep track of where RuleSets are stored in the PDA.  This header is meant
/// to be stored at the beginning of the PDA and never be versioned so that it always
/// has the same serialized size.  See top-level module for description of PDA memory layout.
pub struct RuleSetHeader {
    /// The `Key` for this account which identifies it as a `RuleSet` account.
    pub key: Key,
    /// The location of revision map version stored in the PDA.  This is one byte before the
    /// revision map itself.
    pub rev_map_version_location: usize,
}

impl RuleSetHeader {
    /// Create a new `RuleSetHeader`.
    pub fn new(rev_map_version_location: usize) -> Self {
        Self {
            key: Key::RuleSet,
            rev_map_version_location,
        }
    }
}

/// Size of `RuleSetHeader` when Borsh serialized.
pub const RULE_SET_SERIALIZED_HEADER_LEN: usize = 9;

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone, Default)]
/// Revision map used to keep track of where individual `RuleSet` revisions are stored in the PDA.
/// See top-level module for description of PDA memory layout.
pub struct RuleSetRevisionMapV1 {
    /// `Vec` used to map a `RuleSet` revision number to its location in the PDA.
    pub rule_set_revisions: Vec<usize>,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
/// The struct containing all Rule Set data, most importantly the map of operations to `Rules`.
///  See top-level module for description of PDA memory layout.
pub struct RuleSetV1 {
    /// Version of the RuleSet.  This is not a user version, but the version
    /// of this lib, to make sure that a `RuleSet` passed into our handlers
    /// is one we are compatible with.
    lib_version: u8,
    /// Owner (creator) of the RuleSet.
    #[cfg_attr(feature = "serde-with-feature", serde(with = "As::<DisplayFromStr>"))]
    owner: Pubkey,
    /// Name of the RuleSet, used in PDA derivation.
    rule_set_name: String,
    /// A map to determine the `Rule` that belongs to a given `Operation`.
    // #[serde(deserialize_with = "de_select")]
    pub operations: HashMap<String, Rule>,
}

impl RuleSetV1 {
    /// Create a new empty `RuleSet`.
    pub fn new(rule_set_name: String, owner: Pubkey) -> Self {
        Self {
            lib_version: RULE_SET_LIB_VERSION,
            rule_set_name,
            owner,
            operations: HashMap::new(),
        }
    }

    /// Get the name of the `RuleSet`.
    pub fn name(&self) -> &str {
        &self.rule_set_name
    }

    /// Get the version of the `RuleSet`.
    pub fn lib_version(&self) -> u8 {
        self.lib_version
    }

    /// Get the owner of the `RuleSet`.
    pub fn owner(&self) -> &Pubkey {
        &self.owner
    }

    /// Add a key-value pair into a `RuleSet`.  If this key is already in the `RuleSet`
    /// nothing is updated and an error is returned.
    pub fn add(&mut self, operation: String, rules: Rule) -> ProgramResult {
        if self.operations.get(&operation).is_none() {
            self.operations.insert(operation, rules);
            Ok(())
        } else {
            Err(RuleSetError::ValueOccupied.into())
        }
    }

    /// Retrieve the `Rule` tree for a given `Operation`.
    pub fn get(&self, operation: String) -> Option<&Rule> {
        self.operations.get(&operation)
    }
}

// fn de_select<'de, D>(deserializer: D) -> Result<HashMap<String, Rule>, D::Error>
// where
//     D: serde::de::Deserializer<'de>,
// {
//     deserializer.deserialize_map(MyHmVisitor)
// }

struct MyHmVisitor;

impl<'de> serde::de::Visitor<'de> for MyHmVisitor {
    type Value = HashMap<String, Rule>;

    /// "This is used in error messages. The message should complete the sentence
    /// 'This Visitor expects to receive ...'"
    /// https://docs.serde.rs/src/serde/de/mod.rs.html#1270
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "a HashMap<String, Rule>")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        // extract the size hint from the serialized map. If it doesn't exist, default to 0
        let capacity = map.size_hint().unwrap_or(0);

        let mut hm = HashMap::with_capacity(capacity);

        while let Some((k, v)) = map.next_entry()? {
            solana_program::msg!("k: {}", k);
            hm.insert(k, v);
        }

        Ok(hm)
    }
}

/// Partial deserializer
pub struct PartialSecOpDeserializer {
    op: String,
}

impl PartialSecOpDeserializer {
    /// Create a new `PartialSecOpDeserializer`.
    pub fn new(op: String) -> Self {
        Self { op }
    }
}

impl<'de> Visitor<'de> for PartialSecOpDeserializer {
    type Value = RuleSetV1;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a RuleSetV1")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<RuleSetV1, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        // solana_program::msg!("visit_seq");
        let field0 = match match SeqAccess::next_element::<u8>(&mut seq) {
            Ok(val) => val,
            Err(err) => {
                return Err(err);
            }
        } {
            Some(value) => value,
            None => {
                return Err(serde::de::Error::invalid_length(
                    0usize,
                    &"struct RuleSetV1 with 4 elements",
                ));
            }
        };
        let field1 = match match SeqAccess::next_element::<Pubkey>(&mut seq) {
            Ok(val) => val,
            Err(err) => {
                return Err(err);
            }
        } {
            Some(value) => value,
            None => {
                return Err(serde::de::Error::invalid_length(
                    1usize,
                    &"struct RuleSetV1 with 4 elements",
                ));
            }
        };
        let field2 = match match SeqAccess::next_element::<String>(&mut seq) {
            Ok(val) => val,
            Err(err) => {
                return Err(err);
            }
        } {
            Some(value) => value,
            None => {
                return Err(serde::de::Error::invalid_length(
                    2usize,
                    &"struct RuleSetV1 with 4 elements",
                ));
            }
        };
        // print_type_of(&seq);
        // let pd: PhantomData<u8> = PhantomData;
        // let test = seq.next_element_seed(pd)?;
        // let mut vec = Vec::new();

        let field3 = match match seq.next_element_seed(PartialMapOpDeserializer::new(self.op)) {
            Ok(val) => {
                // solana_program::msg!("val: {:?}", val);
                val
            }
            Err(err) => {
                return Err(err);
            }
        } {
            Some(value) => value,
            None => {
                return Err(serde::de::Error::invalid_length(
                    3usize,
                    &"struct RuleSetV1 with 4 elements",
                ));
            }
        };
        // solana_program::msg!("test: {:?}", test);
        // let field3 = match match SeqAccess::next_element::<HashMap<String, Rule>>(&mut seq) {
        //     Ok(val) => {
        //         solana_program::msg!("val: {:?}", val);
        //         val
        //     }
        //     Err(err) => {
        //         return Err(err);
        //     }
        // } {
        //     Some(value) => value,
        //     None => {
        //         return Err(serde::de::Error::invalid_length(
        //             3usize,
        //             &"struct RuleSetV1 with 4 elements",
        //         ));
        //     }
        // };
        Ok(RuleSetV1 {
            lib_version: field0,
            owner: field1,
            rule_set_name: field2,
            operations: field3,
        })
    }
}

impl<'de> DeserializeSeed<'de> for PartialSecOpDeserializer {
    type Value = RuleSetV1;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        solana_program::msg!("deserialize sec {}", self.op);
        deserializer.deserialize_any(self)
    }
}

/// Partial deserializer
pub struct PartialMapOpDeserializer {
    op: String,
}

impl PartialMapOpDeserializer {
    /// Create a new `PartialMapOpDeserializer`.
    pub fn new(op: String) -> Self {
        Self { op }
    }
}

impl<'de> Visitor<'de> for PartialMapOpDeserializer {
    type Value = HashMap<String, Rule>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "a HashMap<String, Rule>")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        // extract the size hint from the serialized map. If it doesn't exist, default to 0
        let capacity = map.size_hint().unwrap_or(0);

        let mut hm = HashMap::with_capacity(capacity);

        let namespace = self.op.split(':').next().unwrap_or("null");
        solana_program::msg!("namespace: {}", namespace);
        let mut deser_num = 0;
        while let Some((k, v)) = map.next_entry::<String, Rule>()? {
            // solana_program::msg!("o: {:#?}", self.op.as_bytes());
            // solana_program::msg!("k: {:#?}", k.as_bytes());
            // solana_program::msg!("n: {:#?}", namespace.as_bytes());
            if (k == self.op) || (namespace == k) {
                solana_program::msg!("{}", k);
                hm.insert(k, v);
                deser_num += 1;
            } else {
                solana_program::msg!("f@ck off {}", k);
            }
            if deser_num > 1 {
                break;
            }
        }

        Ok(hm)
    }
}

impl<'de> DeserializeSeed<'de> for PartialMapOpDeserializer {
    type Value = HashMap<String, Rule>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        // solana_program::msg!("deserialize map {}", self.op);
        deserializer.deserialize_map(self)
    }
}

// fn print_type_of<T>(_: &T) {
//     solana_program::msg!("{}", std::any::type_name::<T>())
// }
