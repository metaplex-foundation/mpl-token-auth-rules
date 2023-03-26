use borsh::BorshSerialize;
use solana_program::{
    program_error::ProgramError,
    program_memory::sol_memcmp,
    pubkey::{Pubkey, PUBKEY_BYTES},
};

use super::{Constraint, ConstraintType, RuleV2, Str32, U64_BYTES};
use crate::{error::RuleSetError, types::LibVersion};

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
    /// Returns the lib version of the rule set.
    pub fn lib_version(&self) -> u8 {
        (self.header[0] & 0x000000ff) as u8
    }

    /// Returns the name of the rule set.
    pub fn name(&self) -> String {
        self.rule_set_name.to_string()
    }

    /// Returns the number of rules in the rule set.
    pub fn size(&self) -> u32 {
        self.header[1]
    }

    /// Deserialize a constraint from a byte array.
    pub fn from_bytes(bytes: &'a [u8]) -> Result<Self, RuleSetError> {
        // header
        let header = bytemuck::from_bytes::<[u32; 2]>(&bytes[..U64_BYTES]);
        let mut cursor = U64_BYTES;

        // owner
        let owner = bytemuck::from_bytes::<Pubkey>(&bytes[cursor..cursor + PUBKEY_BYTES]);
        cursor += PUBKEY_BYTES;

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

    /// Serialize a `RuleSetV2` into a byte array.
    pub fn serialize(
        owner: Pubkey,
        name: &str,
        operations: &[String],
        rules: &[Vec<u8>],
    ) -> std::io::Result<Vec<u8>> {
        // length of the rule set
        let length = U64_BYTES
            + PUBKEY_BYTES
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

    /// Retrieve the `Rule` tree for a given `Operation`.
    pub fn get(&self, operation: String) -> Option<&RuleV2<'a>> {
        let mut bytes = [0u8; Str32::SIZE];
        bytes[..operation.len()].copy_from_slice(operation.as_bytes());

        for (i, operation) in self.operations.iter().enumerate() {
            if sol_memcmp(&operation.value, &bytes, bytes.len()) == 0 {
                return Some(&self.rules[i]);
            }
        }

        None
    }

    /// This function returns the rule for an operation by recursively searching through fallbacks
    pub fn get_operation(&self, operation: String) -> Result<&RuleV2<'a>, ProgramError> {
        let rule = self.get(operation.to_string());

        match rule {
            Some(rule) => {
                match rule.constraint_type() {
                    ConstraintType::Namespace => {
                        // Check for a ':' namespace separator. If it exists try to operation namespace to see if
                        // a fallback exists. E.g. 'transfer:owner' will check for a fallback for 'transfer'.
                        // If it doesn't exist then fail.
                        let split = operation.split(':').collect::<Vec<&str>>();
                        if split.len() > 1 {
                            self.get_operation(split[0].to_owned())
                        } else {
                            Err(RuleSetError::OperationNotFound.into())
                        }
                    }
                    _ => Ok(rule),
                }
            }
            None => Err(RuleSetError::OperationNotFound.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        state::v2::{Amount, Operator, ProgramOwnedList, RuleSetV2},
        types::LibVersion,
    };
    use solana_program::pubkey::Pubkey;

    #[test]
    fn test_create_amount() {
        // amount rule
        let amount = Amount::serialize(1, Operator::Eq, String::from("Destination")).unwrap();

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

        assert_eq!(rule_set.operations.len(), 2);
        assert_eq!(rule_set.rules.len(), 2);
        assert_eq!(rule_set.lib_version(), LibVersion::V2 as u8);
    }
}
