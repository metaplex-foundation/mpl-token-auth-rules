use borsh::BorshSerialize;
use solana_program::{msg, system_program};
use std::fmt::Display;

use crate::{
    error::RuleSetError,
    state::RuleResult,
    state_v2::{Condition, ConditionType, Str32, HEADER_SECTION},
};

pub struct IsWallet<'a> {
    pub field: &'a Str32,
}

impl<'a> IsWallet<'a> {
    pub fn from_bytes(bytes: &'a [u8]) -> Result<Self, RuleSetError> {
        let field = bytemuck::from_bytes::<Str32>(bytes);

        Ok(Self { field })
    }

    pub fn serialize(field: String) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::with_capacity(HEADER_SECTION + Str32::SIZE);

        // Header
        // - rule type
        let rule_type = ConditionType::PubkeyMatch as u32;
        BorshSerialize::serialize(&rule_type, &mut data)?;
        // - length
        let length = Str32::SIZE as u32;
        BorshSerialize::serialize(&length, &mut data)?;

        // Assert
        // - field
        let mut field_bytes = [0u8; Str32::SIZE];
        field_bytes[..field.len()].copy_from_slice(field.as_bytes());
        BorshSerialize::serialize(&field_bytes, &mut data)?;

        Ok(data)
    }
}

impl<'a> Condition<'a> for IsWallet<'a> {
    fn condition_type(&self) -> ConditionType {
        ConditionType::IsWallet
    }

    fn validate(
        &self,
        accounts: &std::collections::HashMap<
            solana_program::pubkey::Pubkey,
            &solana_program::account_info::AccountInfo,
        >,
        payload: &crate::payload::Payload,
        _update_rule_state: bool,
        _rule_set_state_pda: &Option<&solana_program::account_info::AccountInfo>,
        _rule_authority: &Option<&solana_program::account_info::AccountInfo>,
    ) -> RuleResult {
        msg!("Validating IsWallet");

        // Get the `Pubkey` we are checking from the payload.
        let key = match payload.get_pubkey(&self.field.to_string()) {
            Some(pubkey) => pubkey,
            _ => return RuleResult::Error(RuleSetError::MissingPayloadValue.into()),
        };

        // Get the `AccountInfo` struct for the `Pubkey` and verify that
        // its owner is the System Program.
        if let Some(account) = accounts.get(key) {
            if *account.owner != system_program::ID {
                // TODO: Change error return to commented line after on-curve syscall
                // available.
                return RuleResult::Error(RuleSetError::NotImplemented.into());
                //return (false, self.to_error());
            }
        } else {
            return RuleResult::Error(RuleSetError::MissingAccount.into());
        }

        // TODO: Uncomment call to `is_on_curve()` after on-curve sycall available.
        RuleResult::Error(RuleSetError::NotImplemented.into())
        //(is_on_curve(key), self.to_error())
    }
}

impl<'a> Display for IsWallet<'a> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("IsWallet {")?;
        formatter.write_str(&format!("field: \"{}\"", self.field))?;
        formatter.write_str("}")
    }
}
