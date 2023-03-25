use borsh::BorshSerialize;
use solana_program::{
    msg,
    pubkey::{Pubkey, PUBKEY_BYTES},
};
use std::fmt::Display;

use crate::{
    error::RuleSetError,
    state::RuleResult,
    state_v2::{Condition, ConditionType, Str32, HEADER_SECTION},
};

pub struct PubkeyMatch<'a> {
    pub pubkey: &'a Pubkey,
    pub field: &'a Str32,
}

impl<'a> PubkeyMatch<'a> {
    pub fn from_bytes(bytes: &'a [u8]) -> Result<Self, RuleSetError> {
        let (pubkey, field) = bytes.split_at(PUBKEY_BYTES);
        let pubkey = bytemuck::from_bytes::<Pubkey>(pubkey);
        let field = bytemuck::from_bytes::<Str32>(field);

        Ok(Self { pubkey, field })
    }

    pub fn serialize(pubkey: Pubkey, field: String) -> std::io::Result<Vec<u8>> {
        let length = (PUBKEY_BYTES + Str32::SIZE) as u32;
        let mut data = Vec::with_capacity(HEADER_SECTION + length as usize);

        // Header
        // - rule type
        let rule_type = ConditionType::PubkeyMatch as u32;
        BorshSerialize::serialize(&rule_type, &mut data)?;
        // - length
        BorshSerialize::serialize(&length, &mut data)?;

        // Assert
        // - pubkey
        BorshSerialize::serialize(&pubkey, &mut data)?;
        // - field
        let mut field_bytes = [0u8; Str32::SIZE];
        field_bytes[..field.len()].copy_from_slice(field.as_bytes());
        BorshSerialize::serialize(&field_bytes, &mut data)?;

        Ok(data)
    }
}

impl<'a> Condition<'a> for PubkeyMatch<'a> {
    fn condition_type(&self) -> ConditionType {
        ConditionType::PubkeyMatch
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
        msg!("Validating PubkeyMatch");

        let key = match payload.get_pubkey(&self.field.to_string()) {
            Some(pubkey) => pubkey,
            _ => return RuleResult::Error(RuleSetError::MissingPayloadValue.into()),
        };

        if key == self.pubkey {
            RuleResult::Success(self.condition_type().to_error())
        } else {
            RuleResult::Failure(self.condition_type().to_error())
        }
    }
}

impl<'a> Display for PubkeyMatch<'a> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("PubkeyMatch {pubkey: \"")?;
        formatter.write_str(&format!("{}\",", self.pubkey))?;
        formatter.write_str(&format!("field: \"{}\"", self.field))?;
        formatter.write_str("}")
    }
}
