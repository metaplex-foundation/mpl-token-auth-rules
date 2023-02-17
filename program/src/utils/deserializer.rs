// fn de_select<'de, D>(deserializer: D) -> Result<HashMap<String, Rule>, D::Error>
// where
//     D: serde::de::Deserializer<'de>,
// {
//     deserializer.deserialize_map(MyHmVisitor)
// }

// struct MyHmVisitor;

// impl<'de> serde::de::Visitor<'de> for MyHmVisitor {
//     type Value = HashMap<String, Rule>;

//     /// "This is used in error messages. The message should complete the sentence
//     /// 'This Visitor expects to receive ...'"
//     /// https://docs.serde.rs/src/serde/de/mod.rs.html#1270
//     fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
//         write!(formatter, "a HashMap<String, Rule>")
//     }

//     fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
//     where
//         A: MapAccess<'de>,
//     {
//         // extract the size hint from the serialized map. If it doesn't exist, default to 0
//         let capacity = map.size_hint().unwrap_or(0);

//         let mut hm = HashMap::with_capacity(capacity);

//         while let Some((k, v)) = map.next_entry()? {
//             // solana_program::msg!("k: {}", k);
//             hm.insert(k, v);
//         }

//         Ok(hm)
//     }
// }

use std::collections::HashMap;

use rmp_serde::{
    config::SerializerConfig,
    decode::{Error, ReadSlice},
    Deserializer,
};
use serde::de::{DeserializeSeed, IgnoredAny, Visitor};
use solana_program::pubkey::Pubkey;

use crate::state::{Rule, RuleSetV1};

struct SeqAccess<'a, R, C> {
    de: &'a mut Deserializer<R, C>,
    left: u32,
}

impl<'a, R: 'a, C> SeqAccess<'a, R, C> {
    #[inline]
    fn new(de: &'a mut Deserializer<R, C>, len: u32) -> Self {
        SeqAccess { de, left: len }
    }
}

impl<'de, 'a, R: ReadSlice<'de> + 'a, C: SerializerConfig> serde::de::SeqAccess<'de>
    for SeqAccess<'a, R, C>
{
    type Error = Error;

    #[inline]
    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        if self.left > 0 {
            self.left -= 1;
            Ok(Some(seed.deserialize(&mut *self.de)?))
        } else {
            Ok(None)
        }
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<usize> {
        self.left.try_into().ok()
    }
}

struct MapAccess<'a, R, C> {
    de: &'a mut Deserializer<R, C>,
    left: u32,
}

impl<'a, R: 'a, C> MapAccess<'a, R, C> {
    fn new(de: &'a mut Deserializer<R, C>, len: u32) -> Self {
        MapAccess { de, left: len }
    }
}

impl<'de, 'a, R: ReadSlice<'de> + 'a, C: SerializerConfig> serde::de::MapAccess<'de>
    for MapAccess<'a, R, C>
{
    type Error = Error;

    #[inline]
    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        if self.left > 0 {
            self.left -= 1;
            seed.deserialize(&mut *self.de).map(Some)
        } else {
            Ok(None)
        }
    }

    #[inline]
    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        seed.deserialize(&mut *self.de)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<usize> {
        self.left.try_into().ok()
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
        let field0 = match match seq.next_element::<u8>() {
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
        let field1 = match match seq.next_element::<Pubkey>() {
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
        let field2 = match match seq.next_element::<String>() {
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
        D: serde::Deserializer<'de>,
    {
        // solana_program::msg!("deserialize sec {}", self.op);
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
        // solana_program::msg!("namespace: {}", namespace);
        let mut deser_num = 0;
        while let Some(k) = map.next_key::<String>()? {
            // solana_program::msg!("o: {:#?}", self.op.as_bytes());
            // solana_program::msg!("k: {:#?}", k.as_bytes());
            // solana_program::msg!("n: {:#?}", namespace.as_bytes());
            if (k == self.op) || (namespace == k) {
                let v = map.next_value::<Rule>()?;
                // solana_program::msg!("{}", k);
                hm.insert(k, v);
                deser_num += 1;
            } else {
                // solana_program::msg!("f@ck off {}", k);
                let _ = map.next_value::<IgnoredAny>()?;
            }
            if deser_num == 2 {
                break;
            }
        }

        // Skip over any remaining elements in the sequence after `n`.
        while (map.next_entry::<IgnoredAny, IgnoredAny>()?).is_some() {
            // ignore
        }

        Ok(hm)
    }
}

impl<'de> DeserializeSeed<'de> for PartialMapOpDeserializer {
    type Value = HashMap<String, Rule>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // solana_program::msg!("deserialize map {}", self.op);
        deserializer.deserialize_seq(self)
    }
}
