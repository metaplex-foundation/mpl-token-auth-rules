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

/// Constraint representing the requirement that An additional signer must be present.
///
/// This constraint does not require any `Payload` values, but the additional signer account
/// must be provided to `Validate` via the `additional_rule_accounts` argument so that whether
/// it is a signer can be retrieved from its `AccountInfo` struct.
pub struct AdditionalSigner<'a> {
    /// The public key that must have also signed the transaction.
    pub account: &'a Pubkey,
}

impl<'a> AdditionalSigner<'a> {
    /// Deserialize a constraint from a byte array.
    pub fn from_bytes(bytes: &'a [u8]) -> Result<Self, RuleSetError> {
        let account = try_from_bytes::<Pubkey>(0, PUBKEY_BYTES, bytes)?;
        Ok(Self { account })
    }

    /// Serialize a constraint into a byte array.
    pub fn serialize(account: Pubkey) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::with_capacity(HEADER_SECTION + PUBKEY_BYTES);

        // Header
        // - rule type
        let rule_type = ConstraintType::AdditionalSigner as u32;
        BorshSerialize::serialize(&rule_type, &mut data)?;
        // - length
        let length = PUBKEY_BYTES as u32;
        BorshSerialize::serialize(&length, &mut data)?;

        // Constraint
        // - rule
        BorshSerialize::serialize(&account, &mut data)?;

        Ok(data)
    }
}

impl<'a> Constraint<'a> for AdditionalSigner<'a> {
    fn constraint_type(&self) -> ConstraintType {
        ConstraintType::AdditionalSigner
    }

    fn validate(
        &self,
        accounts: &std::collections::HashMap<
            solana_program::pubkey::Pubkey,
            &solana_program::account_info::AccountInfo,
        >,
        _payload: &crate::payload::Payload,
        _update_rule_state: bool,
        _rule_set_state_pda: &Option<&solana_program::account_info::AccountInfo>,
        _rule_authority: &Option<&solana_program::account_info::AccountInfo>,
    ) -> RuleResult {
        msg!("Validating AdditionalSigner");

        if let Some(signer) = accounts.get(self.account) {
            if signer.is_signer {
                RuleResult::Success(self.constraint_type().to_error())
            } else {
                RuleResult::Failure(self.constraint_type().to_error())
            }
        } else {
            RuleResult::Error(RuleSetError::MissingAccount.into())
        }
    }
}
