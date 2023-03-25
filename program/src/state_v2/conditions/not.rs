use borsh::BorshSerialize;
use solana_program::msg;
use std::fmt::Display;

use crate::{
    error::RuleSetError,
    state::RuleResult,
    state_v2::{Condition, ConditionType, RuleV2, HEADER_SECTION},
};

pub struct Not<'a> {
    pub rule: RuleV2<'a>,
}

impl<'a> Not<'a> {
    pub fn from_bytes(bytes: &'a [u8]) -> Result<Self, RuleSetError> {
        let rule = RuleV2::from_bytes(bytes)?;

        Ok(Self { rule })
    }

    pub fn serialize(rule: &[u8]) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::with_capacity(HEADER_SECTION + rule.len());

        // Header
        // - rule type
        let rule_type = ConditionType::Not as u32;
        BorshSerialize::serialize(&rule_type, &mut data)?;
        // - length
        let length = rule.len() as u32;
        BorshSerialize::serialize(&length, &mut data)?;

        // Assert
        // - rule
        data.extend(rule);

        Ok(data)
    }
}

impl<'a> Condition<'a> for Not<'a> {
    fn condition_type(&self) -> ConditionType {
        ConditionType::Not
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
}

impl<'a> Display for Not<'a> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("Not {rule: [")?;
        formatter.write_str(&format!("{}", self.rule))?;
        formatter.write_str("]}")
    }
}
