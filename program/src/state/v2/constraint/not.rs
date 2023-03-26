use borsh::BorshSerialize;
use solana_program::msg;
use std::fmt::Display;

use crate::{
    error::RuleSetError,
    state::v2::{Constraint, ConstraintType, RuleV2, HEADER_SECTION},
    state::RuleResult,
};

/// Constraint representing a negation, where the contained rule must fail.
pub struct Not<'a> {
    /// The Rule contained under Not.
    pub rule: RuleV2<'a>,
}

impl<'a> Not<'a> {
    /// Deserialize a constraint from a byte array.
    pub fn from_bytes(bytes: &'a [u8]) -> Result<Self, RuleSetError> {
        let rule = RuleV2::from_bytes(bytes)?;

        Ok(Self { rule })
    }

    /// Serialize a constraint into a byte array.
    pub fn serialize(rule: &[u8]) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::with_capacity(HEADER_SECTION + rule.len());

        // Header
        // - rule type
        let rule_type = ConstraintType::Not as u32;
        BorshSerialize::serialize(&rule_type, &mut data)?;
        // - length
        let length = rule.len() as u32;
        BorshSerialize::serialize(&length, &mut data)?;

        // Constraint
        // - rule
        data.extend(rule);

        Ok(data)
    }
}

impl<'a> Constraint<'a> for Not<'a> {
    fn constraint_type(&self) -> ConstraintType {
        ConstraintType::Not
    }

    fn validate(
        &self,
        accounts: &std::collections::HashMap<
            solana_program::pubkey::Pubkey,
            &solana_program::account_info::AccountInfo,
        >,
        payload: &crate::payload::Payload,
        update_rule_state: bool,
        rule_set_state_pda: &Option<&solana_program::account_info::AccountInfo>,
        rule_authority: &Option<&solana_program::account_info::AccountInfo>,
    ) -> RuleResult {
        msg!("Validating Not");

        let result = self.rule.validate(
            accounts,
            payload,
            update_rule_state,
            rule_set_state_pda,
            rule_authority,
        );

        // Negate the result.
        match result {
            RuleResult::Success(err) => RuleResult::Failure(err),
            RuleResult::Failure(err) => RuleResult::Success(err),
            RuleResult::Error(err) => RuleResult::Error(err),
        }
    }

    /// Return a string representation of the constraint.
    fn to_text(&self, indent: usize) -> String {
        let mut output = String::new();
        output.push_str(&format!("{:1$}!", "Any {\n", indent));
        output.push_str(&format!("{:1$}!", "rules: [", indent * 2));
        output.push_str(&self.rule.to_text(indent * 3));
        output.push_str(&format!("{:1$}!", "]", indent * 2));
        output.push_str(&format!("{:1$}!", "}", indent));
        output
    }
}

impl<'a> Display for Not<'a> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.to_text(0))
    }
}
