use std::fmt::Display;

use borsh::BorshSerialize;
use solana_program::pubkey::Pubkey;

use crate::{
    error::RuleSetError,
    state_v2::{AssertType, Assertable, Str32, HEADER_SECTION, SIZE_PUBKEY},
};

pub struct ProgramOwnedList<'a> {
    pub field: &'a [u8],
    pub programs: &'a [Pubkey],
}

impl<'a> ProgramOwnedList<'a> {
    pub fn from_bytes(bytes: &'a [u8]) -> Result<Self, RuleSetError> {
        let (field, programs) = bytes.split_at(Str32::SIZE);
        let field = bytemuck::cast_slice(field);
        let programs = bytemuck::cast_slice(programs);

        Ok(Self { field, programs })
    }

    pub fn serialize(field: String, programs: &[Pubkey]) -> std::io::Result<Vec<u8>> {
        let length = (Str32::SIZE + (programs.len() * SIZE_PUBKEY)) as u32;
        let mut data = Vec::with_capacity(HEADER_SECTION + length as usize);

        // Header
        // - rule type
        let rule_type = AssertType::ProgramOwnedList as u32;
        BorshSerialize::serialize(&rule_type, &mut data)?;
        // - length
        BorshSerialize::serialize(&length, &mut data)?;

        // Assert
        // - field
        let mut field_bytes = [0u8; Str32::SIZE];
        field_bytes[..field.len()].copy_from_slice(field.as_bytes());
        BorshSerialize::serialize(&field_bytes, &mut data)?;
        // - programs
        programs.iter().for_each(|x| {
            BorshSerialize::serialize(x, &mut data).unwrap();
        });

        Ok(data)
    }
}

impl<'a> Assertable<'a> for ProgramOwnedList<'a> {
    fn assert_type(&self) -> AssertType {
        AssertType::ProgramOwnedList
    }
}

impl<'a> Display for ProgramOwnedList<'a> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("ProgramOwnedList {")?;
        formatter.write_str(&format!("programs: [{} pubkeys], ", self.programs.len()))?;
        let field = String::from_utf8(self.field.to_vec()).unwrap();
        formatter.write_str(&format!("field: \"{}\"", field))?;
        formatter.write_str("}")
    }
}
