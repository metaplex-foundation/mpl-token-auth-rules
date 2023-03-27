use borsh::BorshSerialize;
use solana_program::{msg, system_program};

use crate::{
    error::RuleSetError,
    state::v2::{Constraint, ConstraintType, Str32, HEADER_SECTION},
    state::{try_from_bytes, RuleResult},
};

/// Constraint that represents a test on whether a pubkey can be signed from a client and therefore
/// is a true wallet account or not.
///
/// The details of this constraint are as follows: a wallet is defined as being both owned by the
/// System Program and the address is on-curve.  The `field` value in the rule is used to
/// locate the `Pubkey` in the payload that must be on-curve and for which the owner must be
/// the System Program.  Note this same `Pubkey` account must also be provided to `Validate`
/// via the `additional_rule_accounts` argument.  This is so that the `Pubkey`'s owner can be
/// found from its `AccountInfo` struct.
pub struct IsWallet<'a> {
    /// The field in the `Payload` to be checked.
    pub field: &'a Str32,
}

impl<'a> IsWallet<'a> {
    /// Deserialize a constraint from a byte array.
    pub fn from_bytes(bytes: &'a [u8]) -> Result<Self, RuleSetError> {
        let field = try_from_bytes::<Str32>(0, Str32::SIZE, bytes)?;
        Ok(Self { field })
    }

    /// Serialize a constraint into a byte array.
    pub fn serialize(field: String) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::with_capacity(HEADER_SECTION + Str32::SIZE);

        // Header
        // - rule type
        let rule_type = ConstraintType::IsWallet as u32;
        BorshSerialize::serialize(&rule_type, &mut data)?;
        // - length
        let length = Str32::SIZE as u32;
        BorshSerialize::serialize(&length, &mut data)?;

        // Constraint
        // - field
        let mut field_bytes = [0u8; Str32::SIZE];
        field_bytes[..field.len()].copy_from_slice(field.as_bytes());
        BorshSerialize::serialize(&field_bytes, &mut data)?;

        Ok(data)
    }
}

impl<'a> Constraint<'a> for IsWallet<'a> {
    fn constraint_type(&self) -> ConstraintType {
        ConstraintType::IsWallet
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
