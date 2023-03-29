use solana_program::msg;

use crate::{
    error::RuleSetError,
    state::v2::{Constraint, ConstraintType, HEADER_SECTION},
    state::{Header, RuleResult},
};

/// A constraint that tells the operation finder to use the default namespace rule.
pub struct Namespace;

impl<'a> Namespace {
    /// Deserialize a constraint from a byte array.
    pub fn from_bytes(_bytes: &'a [u8]) -> Result<Self, RuleSetError> {
        Ok(Self {})
    }

    /// Serialize a constraint into a byte array.
    pub fn serialize() -> Result<Vec<u8>, RuleSetError> {
        let mut data = Vec::with_capacity(HEADER_SECTION);
        // Header
        Header::serialize(ConstraintType::Namespace, 0, &mut data);

        Ok(data)
    }
}

impl<'a> Constraint<'a> for Namespace {
    fn constraint_type(&self) -> ConstraintType {
        ConstraintType::Namespace
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
        _rule_authority: &Option<&solana_program::account_info::AccountInfo>,
    ) -> RuleResult {
        msg!("Validating Namespace");
        // should never be called directly
        RuleResult::Failure(self.constraint_type().to_error())
    }
}
