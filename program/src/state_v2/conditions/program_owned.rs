use borsh::BorshSerialize;
use solana_program::{
    msg,
    program_error::ProgramError,
    pubkey::{Pubkey, PUBKEY_BYTES},
};
use std::fmt::Display;

use crate::{
    error::RuleSetError,
    state::RuleResult,
    state_v2::{Condition, ConditionType, Str32, HEADER_SECTION},
    utils::is_zeroed,
};

pub struct ProgramOwned<'a> {
    pub program: &'a Pubkey,
    pub field: &'a Str32,
}

impl<'a> ProgramOwned<'a> {
    pub fn from_bytes(bytes: &'a [u8]) -> Result<Self, RuleSetError> {
        let (program, field) = bytes.split_at(PUBKEY_BYTES);
        let program = bytemuck::from_bytes::<Pubkey>(program);
        let field = bytemuck::from_bytes::<Str32>(field);

        Ok(Self { program, field })
    }

    pub fn serialize(program: Pubkey, field: String) -> std::io::Result<Vec<u8>> {
        let length = (PUBKEY_BYTES + Str32::SIZE) as u32;
        let mut data = Vec::with_capacity(HEADER_SECTION + length as usize);

        // Header
        // - rule type
        let rule_type = ConditionType::PubkeyMatch as u32;
        BorshSerialize::serialize(&rule_type, &mut data)?;
        // - length
        BorshSerialize::serialize(&length, &mut data)?;

        // Assert
        // - program
        BorshSerialize::serialize(&program, &mut data)?;
        // - field
        let mut field_bytes = [0u8; Str32::SIZE];
        field_bytes[..field.len()].copy_from_slice(field.as_bytes());
        BorshSerialize::serialize(&field_bytes, &mut data)?;

        Ok(data)
    }
}

impl<'a> Condition<'a> for ProgramOwned<'a> {
    fn condition_type(&self) -> ConditionType {
        ConditionType::ProgramOwned
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
        msg!("Validating ProgramOwned");

        let key = match payload.get_pubkey(&self.field.to_string()) {
            Some(pubkey) => pubkey,
            _ => return RuleResult::Error(RuleSetError::MissingPayloadValue.into()),
        };

        if let Some(account) = accounts.get(key) {
            let data = match account.data.try_borrow() {
                Ok(data) => data,
                Err(_) => return RuleResult::Error(ProgramError::AccountBorrowFailed),
            };

            if is_zeroed(&data) {
                // Print helpful errors.
                if data.len() == 0 {
                    msg!("Account data is empty");
                } else {
                    msg!("Account data is zeroed");
                }

                // Account must have nonzero data to count as program-owned.
                return RuleResult::Error(self.condition_type().to_error());
            } else if *account.owner == *self.program {
                return RuleResult::Success(self.condition_type().to_error());
            }
        } else {
            return RuleResult::Error(RuleSetError::MissingAccount.into());
        }

        RuleResult::Failure(self.condition_type().to_error())
    }
}

impl<'a> Display for ProgramOwned<'a> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("ProgramOwned {program: \"")?;
        formatter.write_str(&format!("{}\",", self.program))?;
        formatter.write_str(&format!("field: \"{}\"", self.field))?;
        formatter.write_str("}")
    }
}
