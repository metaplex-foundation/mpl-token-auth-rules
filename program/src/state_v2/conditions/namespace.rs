use borsh::BorshSerialize;
use solana_program::msg;
use std::fmt::Display;

use crate::{
    error::RuleSetError,
    state::RuleResult,
    state_v2::{Condition, ConditionType, HEADER_SECTION},
};

pub struct Namespace {}

impl<'a> Namespace {
    pub fn from_bytes(_bytes: &'a [u8]) -> Result<Self, RuleSetError> {
        Ok(Self {})
    }

    pub fn serialize() -> std::io::Result<Vec<u8>> {
        let mut data = Vec::with_capacity(HEADER_SECTION);

        // Header
        // - rule type
        let rule_type = ConditionType::Any as u32;
        BorshSerialize::serialize(&rule_type, &mut data)?;
        // - length
        BorshSerialize::serialize(&0u32, &mut data)?;

        Ok(data)
    }
}

impl<'a> Condition<'a> for Namespace {
    fn condition_type(&self) -> ConditionType {
        ConditionType::Namespace
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
        RuleResult::Failure(self.condition_type().to_error())
    }
}

impl Display for Namespace {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("Namespace")
    }
}
