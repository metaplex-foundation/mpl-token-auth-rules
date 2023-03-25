use std::fmt::Display;

use borsh::BorshSerialize;
use solana_program::{
    msg,
    program_error::ProgramError,
    pubkey::{Pubkey, PUBKEY_BYTES},
};

use crate::{
    error::RuleSetError,
    state::RuleResult,
    state_v2::{Condition, ConditionType, Str32, HEADER_SECTION},
    utils::is_zeroed,
};

pub struct ProgramOwnedList<'a> {
    pub field: &'a Str32,
    pub programs: &'a [Pubkey],
}

impl<'a> ProgramOwnedList<'a> {
    pub fn from_bytes(bytes: &'a [u8]) -> Result<Self, RuleSetError> {
        let (field, programs) = bytes.split_at(Str32::SIZE);
        let field = bytemuck::from_bytes::<Str32>(field);
        let programs = bytemuck::cast_slice(programs);

        Ok(Self { field, programs })
    }

    pub fn serialize(field: String, programs: &[Pubkey]) -> std::io::Result<Vec<u8>> {
        let length = (Str32::SIZE + (programs.len() * PUBKEY_BYTES)) as u32;
        let mut data = Vec::with_capacity(HEADER_SECTION + length as usize);

        // Header
        // - rule type
        let rule_type = ConditionType::ProgramOwnedList as u32;
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

impl<'a> Condition<'a> for ProgramOwnedList<'a> {
    fn condition_type(&self) -> ConditionType {
        ConditionType::ProgramOwnedList
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
        msg!("Validating ProgramOwnedList");

        let field = self.field.to_string();

        for field in field.split('|') {
            let key = match payload.get_pubkey(&field.to_string()) {
                Some(pubkey) => pubkey,
                _ => return RuleResult::Error(RuleSetError::MissingPayloadValue.into()),
            };

            let account = match accounts.get(key) {
                Some(account) => account,
                _ => return RuleResult::Error(RuleSetError::MissingAccount.into()),
            };

            let data = match account.data.try_borrow() {
                Ok(data) => data,
                Err(_) => return RuleResult::Error(ProgramError::AccountBorrowFailed),
            };

            if is_zeroed(&data) {
                // Print helpful errors.
                msg!(if data.len() == 0 {
                    "Account data is empty"
                } else {
                    "Account data is zeroed"
                });

                return RuleResult::Error(RuleSetError::DataIsEmpty.into());
            } else if self.programs.contains(account.owner) {
                // Account owner must be in the set.
                return RuleResult::Success(self.condition_type().to_error());
            }
        }

        RuleResult::Failure(self.condition_type().to_error())
    }
}

impl<'a> Display for ProgramOwnedList<'a> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("ProgramOwnedList {")?;
        formatter.write_str(&format!("programs: [{} pubkeys], ", self.programs.len()))?;
        formatter.write_str(&format!("field: \"{}\"", self.field))?;
        formatter.write_str("}")
    }
}
