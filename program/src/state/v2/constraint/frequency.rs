use borsh::BorshSerialize;
use solana_program::{
    msg,
    pubkey::{Pubkey, PUBKEY_BYTES},
};

use crate::{
    error::RuleSetError,
    state::v2::{Constraint, ConstraintType, HEADER_SECTION},
    state::{try_from_bytes, RuleResult},
};

/// Constraint representing a comparison based on time between operations.
///
/// Currently not implemented. This constraint is planned check to ensure a certain
/// amount of time has passed.  This rule will make use of the `rule_set_state_pda`
/// optional account passed into `Validate`, and will require the optional
/// `rule_authority` account to sign.
pub struct Frequency<'a> {
    /// The authority of the frequency account.
    pub authority: &'a Pubkey,
}

impl<'a> Frequency<'a> {
    /// Deserialize a constraint from a byte array.
    pub fn from_bytes(bytes: &'a [u8]) -> Result<Self, RuleSetError> {
        let authority = try_from_bytes::<Pubkey>(0, PUBKEY_BYTES, bytes)?;
        Ok(Self { authority })
    }

    /// Serialize a constraint into a byte array.
    pub fn serialize(authority: Pubkey) -> std::io::Result<Vec<u8>> {
        let length = PUBKEY_BYTES as u32;
        let mut data = Vec::with_capacity(HEADER_SECTION + length as usize);

        // Header
        // - rule type
        let rule_type = ConstraintType::Frequency as u32;
        BorshSerialize::serialize(&rule_type, &mut data)?;
        // - length
        BorshSerialize::serialize(&length, &mut data)?;

        // Constraint
        // - pubkey
        BorshSerialize::serialize(&authority, &mut data)?;

        Ok(data)
    }
}

impl<'a> Constraint<'a> for Frequency<'a> {
    fn constraint_type(&self) -> ConstraintType {
        ConstraintType::Frequency
    }

    fn validate(
        &self,
        _accounts: &std::collections::HashMap<
            solana_program::pubkey::Pubkey,
            &solana_program::account_info::AccountInfo,
        >,
        _payload: &crate::payload::Payload,
        _update_rule_state: bool,
        _rule_set_state_pda: &Option<&solana_program::account_info::AccountInfo>,
        rule_authority: &Option<&solana_program::account_info::AccountInfo>,
    ) -> RuleResult {
        msg!("Validating Frequency");

        if let Some(rule_authority) = rule_authority {
            // TODO: If it's the wrong account (first condition) the `IsNotASigner`
            // is misleading.  Should be improved, perhaps with a `Mismatch` error.
            if self.authority != rule_authority.key || !rule_authority.is_signer {
                return RuleResult::Error(RuleSetError::RuleAuthorityIsNotSigner.into());
            }
        } else {
            return RuleResult::Error(RuleSetError::MissingAccount.into());
        }

        RuleResult::Error(RuleSetError::NotImplemented.into())
    }
}
