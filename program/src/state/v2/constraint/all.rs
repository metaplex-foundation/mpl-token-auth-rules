use borsh::BorshSerialize;
use solana_program::{msg, program_error::ProgramError};
use std::fmt::Display;

use crate::{
    error::RuleSetError,
    state::v2::{Constraint, ConstraintType, RuleV2, HEADER_SECTION, U64_BYTES},
    state::RuleResult,
};

/// Constraint representing a group AND, where every rule contained must pass.
pub struct All<'a> {
    /// The number of rules contained under All.
    pub size: &'a u64,
    /// The vector of 'RuleV2' contained under All.
    pub rules: Vec<RuleV2<'a>>,
}

impl<'a> All<'a> {
    /// Deserialize a constraint from a byte array.
    pub fn from_bytes(bytes: &'a [u8]) -> Result<Self, RuleSetError> {
        let (size, data) = bytes.split_at(U64_BYTES);
        let size = bytemuck::from_bytes::<u64>(size);

        let mut rules = Vec::with_capacity(*size as usize);
        let mut offset = 0;

        for _ in 0..*size {
            let rule = RuleV2::from_bytes(&data[offset..])?;
            offset += rule.length();
            rules.push(rule);
        }

        Ok(Self { size, rules })
    }

    /// Serialize a constraint into a byte array.
    pub fn serialize(rules: &[&[u8]]) -> std::io::Result<Vec<u8>> {
        // length of the assert
        let length = (U64_BYTES
            + rules
                .iter()
                .map(|v| v.len())
                .reduce(|accum, item| accum + item)
                .ok_or(RuleSetError::DataIsEmpty)
                .unwrap()) as u32;

        let mut data = Vec::with_capacity(HEADER_SECTION + length as usize);

        // Header
        // - rule type
        let rule_type = ConstraintType::All as u32;
        BorshSerialize::serialize(&rule_type, &mut data)?;
        // - length
        BorshSerialize::serialize(&length, &mut data)?;

        // Constraint
        // - size
        let size = rules.len() as u64;
        BorshSerialize::serialize(&size, &mut data)?;
        // - rules
        rules.iter().for_each(|x| data.extend(x.iter()));

        Ok(data)
    }
}

impl<'a> Constraint<'a> for All<'a> {
    fn constraint_type(&self) -> ConstraintType {
        ConstraintType::All
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
        msg!("Validating All");

        let mut last: Option<ProgramError> = None;

        for rule in &self.rules {
            let result = rule.validate(
                accounts,
                payload,
                update_rule_state,
                rule_set_state_pda,
                rule_authority,
            );
            // Return failure on the first failing rule.
            match result {
                RuleResult::Success(err) => last = Some(err),
                _ => return result,
            }
        }

        // Return pass if and only if all rules passed.
        RuleResult::Success(last.unwrap_or_else(|| RuleSetError::UnexpectedRuleSetFailure.into()))
    }

    /// Return a string representation of the constraint.
    fn to_text(&self, indent: usize) -> String {
        let mut output = String::new();
        output.push_str(&format!("{:1$}!", "All {\n", indent));
        output.push_str(&format!("{:1$}!", "rules: [", indent * 2));

        for i in 0..*self.size {
            output.push_str(&format!(
                "{}{}",
                self.rules[i as usize].to_text(indent + 3),
                if i > 0 { ", " } else { "" },
            ));
        }

        output.push_str(&format!("{:1$}!", "]", indent * 2));
        output.push_str(&format!("{:1$}!", "}", indent));
        output
    }
}

impl<'a> Display for All<'a> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.to_text(0))
    }
}
