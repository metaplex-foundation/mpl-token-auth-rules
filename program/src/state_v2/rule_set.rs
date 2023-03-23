use std::fmt::Display;

use borsh::BorshSerialize;
use solana_program::pubkey::Pubkey;

use super::{RuleV2, Str32, SIZE_PUBKEY, SIZE_U64};
use crate::{error::RuleSetError, LibVersion, MAX_NAME_LENGTH};

/// The struct containing all Rule Set data, most importantly the map of operations to `Rules`.
///  See top-level module for description of PDA memory layout.
pub struct RuleSetV2<'a> {
    /// Header information. The first byte holds the lib_version of the rule set
    /// and the last 4 bytes (u32) represent the number of rules.
    header: &'a [u32; 2],

    /// Owner (creator) of the RuleSet.
    pub owner: &'a Pubkey,

    /// Name of the RuleSet, used in PDA derivation.
    pub rule_set_name: &'a Str32,

    /// Operations available.
    pub operations: &'a [Str32],

    /// Rules for each operation.
    pub rules: Vec<RuleV2<'a>>,
}

impl<'a> RuleSetV2<'a> {
    pub fn lib_version(&self) -> u8 {
        (self.header[0] & 0x000000ff) as u8
    }

    pub fn name(&self) -> String {
        let end_index = self
            .rule_set_name
            .value
            .iter()
            .position(|&x| x == b'\0')
            .unwrap_or(MAX_NAME_LENGTH);
        // return a copy of the name without any padding bytes
        String::from_utf8_lossy(&self.rule_set_name.value[..end_index]).to_string()
    }

    pub fn size(&self) -> u32 {
        self.header[1]
    }

    pub fn from_bytes(bytes: &'a [u8]) -> Result<Self, RuleSetError> {
        // header
        let header = bytemuck::from_bytes::<[u32; 2]>(&bytes[..SIZE_U64]);
        let mut cursor = SIZE_U64;

        // owner
        let owner = bytemuck::from_bytes::<Pubkey>(&bytes[cursor..cursor + SIZE_PUBKEY]);
        cursor += SIZE_PUBKEY;

        // name
        let rule_set_name = bytemuck::from_bytes::<Str32>(&bytes[cursor..cursor + Str32::SIZE]);
        cursor += Str32::SIZE;

        // number of operations and rules
        let size = header[1] as usize;

        // operations
        let slice_end = cursor
            + Str32::SIZE
                .checked_mul(size)
                .ok_or(RuleSetError::NumericalOverflow)?;
        let operations = bytemuck::cast_slice(&bytes[cursor..slice_end]);
        cursor = slice_end;

        // rules
        let mut rules = Vec::with_capacity(size);

        for _ in 0..size {
            let rule = RuleV2::from_bytes(&bytes[cursor..]).unwrap();
            cursor += rule.length();
            rules.push(rule);
        }

        Ok(Self {
            header,
            owner,
            rule_set_name,
            operations,
            rules,
        })
    }

    pub fn serialize(
        owner: Pubkey,
        name: &str,
        operations: &[String],
        rules: &[Vec<u8>],
    ) -> std::io::Result<Vec<u8>> {
        // length of the rule set
        let length = SIZE_U64
            + SIZE_PUBKEY
            + Str32::SIZE
            + (operations.len() * Str32::SIZE)
            + rules
                .iter()
                .map(|v| v.len())
                .reduce(|accum, item| accum + item)
                .ok_or(RuleSetError::DataIsEmpty)
                .unwrap();

        let mut data = Vec::with_capacity(length);

        // header
        // - lib version
        let lib_version = u32::from_le_bytes([LibVersion::V2 as u8, 0, 0, 0]);
        BorshSerialize::serialize(&lib_version, &mut data)?;
        // - size
        let size = operations.len() as u32;
        BorshSerialize::serialize(&size, &mut data)?;

        // owner
        BorshSerialize::serialize(&owner, &mut data)?;

        // name
        let mut field_bytes = [0u8; Str32::SIZE];
        field_bytes[..name.len()].copy_from_slice(name.as_bytes());
        BorshSerialize::serialize(&field_bytes, &mut data)?;

        // operations
        operations.iter().for_each(|x| {
            let mut field_bytes = [0u8; Str32::SIZE];
            field_bytes[..x.len()].copy_from_slice(x.as_bytes());
            BorshSerialize::serialize(&field_bytes, &mut data).unwrap();
        });

        // rules
        rules.iter().for_each(|x| data.extend(x.iter()));

        Ok(data)
    }
}

impl<'a> Display for RuleSetV2<'a> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&format!("RuleSet: {} {{", self.rule_set_name))?;
        formatter.write_str("operations: [")?;

        for i in 0..self.size() {
            if i > 0 {
                formatter.write_str(", ")?;
            }
            formatter.write_str(&format!(
                "\"{}\": {:}",
                self.operations[i as usize], self.rules[i as usize]
            ))?;
        }

        formatter.write_str("]}")
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        state_v2::{Amount, CompareOp, ProgramOwnedList, RuleSetV2},
        LibVersion,
    };
    use solana_program::pubkey::Pubkey;

    #[test]
    fn test_create_amount() {
        // amount rule
        let amount = Amount::serialize(1, CompareOp::Eq, String::from("Destination")).unwrap();

        // program owned rule
        let programs = &[Pubkey::default(), Pubkey::default()];

        let program_owned =
            ProgramOwnedList::serialize(String::from("Destination"), programs).unwrap();

        // rule set

        let serialized = RuleSetV2::serialize(
            Pubkey::default(),
            "Royalties",
            &["deletage_transfer".to_string(), "transfer".to_string()],
            &[amount, program_owned],
        )
        .unwrap();

        // loads a rule set object

        let rule_set = RuleSetV2::from_bytes(&serialized).unwrap();
        println!("{}", rule_set);

        assert_eq!(rule_set.operations.len(), 2);
        assert_eq!(rule_set.rules.len(), 2);
        assert_eq!(rule_set.lib_version(), LibVersion::V2 as u8);
    }
}
