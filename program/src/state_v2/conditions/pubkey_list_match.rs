use std::fmt::Display;

use borsh::BorshSerialize;
use solana_program::{
    msg,
    pubkey::{Pubkey, PUBKEY_BYTES},
};

use crate::{
    error::RuleSetError,
    state::RuleResult,
    state_v2::{Condition, ConditionType, Str32, HEADER_SECTION},
};

pub struct PubkeyListMatch<'a> {
    pub field: &'a Str32,
    pub pubkeys: &'a [Pubkey],
}

impl<'a> PubkeyListMatch<'a> {
    pub fn from_bytes(bytes: &'a [u8]) -> Result<Self, RuleSetError> {
        let (field, pubkeys) = bytes.split_at(Str32::SIZE);
        let field = bytemuck::from_bytes::<Str32>(field);
        let pubkeys = bytemuck::cast_slice(pubkeys);

        Ok(Self { field, pubkeys })
    }

    pub fn serialize(field: String, pubkeys: &[Pubkey]) -> std::io::Result<Vec<u8>> {
        let length = (Str32::SIZE + (pubkeys.len() * PUBKEY_BYTES)) as u32;
        let mut data = Vec::with_capacity(HEADER_SECTION + length as usize);

        // Header
        // - rule type
        let rule_type = ConditionType::PubkeyListMatch as u32;
        BorshSerialize::serialize(&rule_type, &mut data)?;
        // - length
        BorshSerialize::serialize(&length, &mut data)?;

        // Assert
        // - field
        let mut field_bytes = [0u8; Str32::SIZE];
        field_bytes[..field.len()].copy_from_slice(field.as_bytes());
        BorshSerialize::serialize(&field_bytes, &mut data)?;
        // - programs
        pubkeys.iter().for_each(|x| {
            BorshSerialize::serialize(x, &mut data).unwrap();
        });

        Ok(data)
    }
}

impl<'a> Condition<'a> for PubkeyListMatch<'a> {
    fn condition_type(&self) -> ConditionType {
        ConditionType::PubkeyListMatch
    }

    fn validate(
        &self,
        _accounts: &std::collections::HashMap<
            solana_program::pubkey::Pubkey,
            &solana_program::account_info::AccountInfo,
        >,
        payload: &crate::payload::Payload,
        _update_rule_state: bool,
        _rule_set_state_pda: &Option<&solana_program::account_info::AccountInfo>,
        _rule_authority: &Option<&solana_program::account_info::AccountInfo>,
    ) -> RuleResult {
        msg!("Validating PubkeyListMatch");

        let field = self.field.to_string();

        for field in field.split('|') {
            let key = match payload.get_pubkey(&field.to_string()) {
                Some(pubkey) => pubkey,
                _ => return RuleResult::Error(RuleSetError::MissingPayloadValue.into()),
            };

            if self.pubkeys.contains(key) {
                // Account owner must be in the set.
                return RuleResult::Success(self.condition_type().to_error());
            }
        }

        RuleResult::Failure(self.condition_type().to_error())
    }
}

impl<'a> Display for PubkeyListMatch<'a> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("ProgramOwnedList {")?;
        formatter.write_str(&format!("pubkeys: [{} pubkeys], ", self.pubkeys.len()))?;
        formatter.write_str(&format!("field: \"{}\"", self.field))?;
        formatter.write_str("}")
    }
}
