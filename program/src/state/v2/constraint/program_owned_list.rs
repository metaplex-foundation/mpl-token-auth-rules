use borsh::BorshSerialize;
use solana_program::{
    msg,
    program_error::ProgramError,
    pubkey::{Pubkey, PUBKEY_BYTES},
};

use crate::{
    error::RuleSetError,
    state::RuleResult,
    state::{
        try_cast_slice, try_from_bytes,
        v2::{Constraint, ConstraintType, Str32, HEADER_SECTION},
    },
    utils::is_zeroed,
};

/// Constraint representing a test where the `Pubkey` must be owned by a program in the list of `Pubkey`s.
///
/// This constraint requires a `PayloadType` value of `PayloadType::Pubkey`. The `field` value in the
/// rule is used to locate the `Pubkey` in the payload for which the owner must be a program in the list
/// in the rule.  Note this same `Pubkey` account must also be provided to `Validate` via the
/// `additional_rule_accounts` argument.  This is so that the `Pubkey`'s owner can be found from its
/// `AccountInfo` struct.
pub struct ProgramOwnedList<'a> {
    /// The field in the `Payload` to be compared.
    pub field: &'a Str32,
    /// The program that must own the `Pubkey`.
    pub programs: &'a [Pubkey],
}

impl<'a> ProgramOwnedList<'a> {
    /// Deserialize a constraint from a byte array.
    pub fn from_bytes(bytes: &'a [u8]) -> Result<Self, RuleSetError> {
        let field = try_from_bytes::<Str32>(0, Str32::SIZE, bytes)?;
        let programs = try_cast_slice(&bytes[Str32::SIZE..])?;

        Ok(Self { field, programs })
    }

    /// Serialize a constraint into a byte array.
    pub fn serialize(field: String, programs: &[Pubkey]) -> std::io::Result<Vec<u8>> {
        let length = (Str32::SIZE + (programs.len() * PUBKEY_BYTES)) as u32;
        let mut data = Vec::with_capacity(HEADER_SECTION + length as usize);

        // Header
        // - rule type
        let rule_type = ConstraintType::ProgramOwnedList as u32;
        BorshSerialize::serialize(&rule_type, &mut data)?;
        // - length
        BorshSerialize::serialize(&length, &mut data)?;

        // Constraint
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

impl<'a> Constraint<'a> for ProgramOwnedList<'a> {
    fn constraint_type(&self) -> ConstraintType {
        ConstraintType::ProgramOwnedList
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
                return RuleResult::Success(self.constraint_type().to_error());
            }
        }

        RuleResult::Failure(self.constraint_type().to_error())
    }
}
