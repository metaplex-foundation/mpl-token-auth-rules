use solana_program::{msg, program_error::ProgramError};

use crate::{
    error::RuleSetError,
    state::{try_from_bytes, RuleResult},
    state::{
        v2::{Constraint, ConstraintType, RuleV2, HEADER_SECTION, U64_BYTES},
        Header,
    },
};

/// Constraint representing a group OR, where at least one rule contained must pass.
pub struct Any<'a> {
    /// The number of rules contained under Any.
    pub size: &'a u64,
    /// The vector of Rules contained under Any.
    pub rules: Vec<RuleV2<'a>>,
}

impl<'a> Any<'a> {
    /// Deserialize a constraint from a byte array.
    pub fn from_bytes(bytes: &'a [u8]) -> Result<Self, RuleSetError> {
        let size = try_from_bytes::<u64>(0, U64_BYTES, bytes)?;

        let mut rules = Vec::with_capacity(*size as usize);
        let mut offset = U64_BYTES;

        for _ in 0..*size {
            let rule = RuleV2::from_bytes(&bytes[offset..])?;
            offset += rule.length();
            rules.push(rule);
        }

        Ok(Self { size, rules })
    }

    /// Serialize a constraint into a byte array.
    pub fn serialize(rules: &[&[u8]]) -> Result<Vec<u8>, RuleSetError> {
        let length = (U64_BYTES
            + rules
                .iter()
                .map(|v| v.len())
                .reduce(|accum, item| accum + item)
                .ok_or(RuleSetError::DataIsEmpty)
                .unwrap()) as u32;

        let mut data = Vec::with_capacity(HEADER_SECTION + length as usize);

        // Header
        Header::serialize(ConstraintType::Any, length, &mut data);

        // Constraint
        // - size
        data.extend(u64::to_le_bytes(rules.len() as u64));
        // - rules
        rules.iter().for_each(|x| data.extend(x.iter()));

        Ok(data)
    }
}

impl<'a> Constraint<'a> for Any<'a> {
    fn constraint_type(&self) -> ConstraintType {
        ConstraintType::Any
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
        msg!("Validating Any");

        let mut last_failure: Option<ProgramError> = None;
        let mut last_error: Option<ProgramError> = None;

        for rule in &self.rules {
            let result = rule.validate(
                accounts,
                payload,
                update_rule_state,
                rule_set_state_pda,
                rule_authority,
            );

            match result {
                RuleResult::Success(_) => return result,
                RuleResult::Failure(err) => last_failure = Some(err),
                RuleResult::Error(err) => last_error = Some(err),
            }
        }

        // Return the last failure if and only if no rules passed and there was at least one failure,
        // otherwise return the last error

        if let Some(err) = last_failure {
            RuleResult::Failure(err)
        } else if let Some(err) = last_error {
            RuleResult::Error(err)
        } else {
            RuleResult::Error(RuleSetError::UnexpectedRuleSetFailure.into())
        }
    }
}
