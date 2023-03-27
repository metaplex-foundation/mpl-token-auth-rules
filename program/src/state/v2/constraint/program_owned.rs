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
        try_from_bytes,
        v2::{Constraint, ConstraintType, Str32, HEADER_SECTION},
    },
    utils::is_zeroed,
};

/// Constraint representing a test where a `Pubkey` must be owned by a given program.
///
/// This constraint requires a `PayloadType` value of `PayloadType::Pubkey`.  The `field` value in
/// the rule is used to locate the `Pubkey` in the payload for which the owner must be the
/// program in the rule.  Note this same `Pubkey` account must also be provided to `Validate`
/// via the `additional_rule_accounts` argument.  This is so that the `Pubkey`'s owner can be
/// found from its `AccountInfo` struct.
pub struct ProgramOwned<'a> {
    /// The program that must own the `Pubkey`.
    pub program: &'a Pubkey,
    /// The field in the `Payload` to be compared.
    pub field: &'a Str32,
}

impl<'a> ProgramOwned<'a> {
    /// Deserialize a constraint from a byte array.
    pub fn from_bytes(bytes: &'a [u8]) -> Result<Self, RuleSetError> {
        let program = try_from_bytes::<Pubkey>(0, PUBKEY_BYTES, bytes)?;
        let field = try_from_bytes::<Str32>(PUBKEY_BYTES, Str32::SIZE, bytes)?;

        Ok(Self { program, field })
    }

    /// Serialize a constraint into a byte array.
    pub fn serialize(program: Pubkey, field: String) -> std::io::Result<Vec<u8>> {
        let length = (PUBKEY_BYTES + Str32::SIZE) as u32;
        let mut data = Vec::with_capacity(HEADER_SECTION + length as usize);

        // Header
        // - rule type
        let rule_type = ConstraintType::ProgramOwned as u32;
        BorshSerialize::serialize(&rule_type, &mut data)?;
        // - length
        BorshSerialize::serialize(&length, &mut data)?;

        // Constraint
        // - program
        BorshSerialize::serialize(&program, &mut data)?;
        // - field
        let mut field_bytes = [0u8; Str32::SIZE];
        field_bytes[..field.len()].copy_from_slice(field.as_bytes());
        BorshSerialize::serialize(&field_bytes, &mut data)?;

        Ok(data)
    }
}

impl<'a> Constraint<'a> for ProgramOwned<'a> {
    fn constraint_type(&self) -> ConstraintType {
        ConstraintType::ProgramOwned
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
                return RuleResult::Error(self.constraint_type().to_error());
            } else if *account.owner == *self.program {
                return RuleResult::Success(self.constraint_type().to_error());
            }
        } else {
            return RuleResult::Error(RuleSetError::MissingAccount.into());
        }

        RuleResult::Failure(self.constraint_type().to_error())
    }
}
