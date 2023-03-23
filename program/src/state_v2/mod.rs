pub mod conditions;
pub mod rule;
pub mod rule_set;

use bytemuck::{Pod, Zeroable};
use solana_program::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey};
use std::{collections::HashMap, fmt::Display};

pub use conditions::*;
pub use rule::*;
pub use rule_set::*;

use crate::{error::RuleSetError, payload::Payload, state::RuleResult, types::MAX_NAME_LENGTH};

// Size of a u64 value.
pub const U64_BYTES: usize = std::mem::size_of::<u64>();

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct Str32 {
    pub value: [u8; MAX_NAME_LENGTH],
}

impl Str32 {
    pub const SIZE: usize = MAX_NAME_LENGTH;
}

impl Display for Str32 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = String::from_utf8(self.value.to_vec()).unwrap();
        formatter.write_str(value.as_str())
    }
}

pub trait Condition<'a>: Display {
    fn validate(
        &self,
        _accounts: &HashMap<Pubkey, &AccountInfo>,
        _payload: &Payload,
        _update_rule_state: bool,
        _rule_set_state_pda: &Option<&AccountInfo>,
        _rule_authority: &Option<&AccountInfo>,
    ) -> RuleResult {
        RuleResult::Success(self.condition_type().to_error())
    }

    fn condition_type(&self) -> ConditionType;
}

#[repr(u64)]
#[derive(PartialEq, Eq, Debug, Clone)]
/// Operators that can be used to compare against an `Amount` rule.
pub enum CompareOp {
    /// Less Than
    Lt,
    /// Less Than or Equal To
    LtEq,
    /// Equal To
    Eq,
    /// Greater Than or Equal To
    GtEq,
    /// Greater Than
    Gt,
}

impl TryFrom<u64> for CompareOp {
    // Type of the error generated.
    type Error = RuleSetError;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(CompareOp::Lt),
            1 => Ok(CompareOp::LtEq),
            2 => Ok(CompareOp::Eq),
            3 => Ok(CompareOp::GtEq),
            4 => Ok(CompareOp::Gt),
            value => {
                panic!("invalid operator: {}", value)
            }
        }
    }
}

#[repr(u32)]
#[derive(Clone, Copy)]
/// The struct containing every type of Rule and its associated data.
pub enum ConditionType {
    /// Group AND, where every rule contained must pass.
    All,
    /// Group OR, where at least one rule contained must pass.
    Any,
    /// Negation, where the contained rule must fail.
    ProgramOwnedList,
    /// Comparison against the amount of tokens being transferred.   When the `Validate`
    /// instruction is called, this rule requires a `PayloadType` value of `PayloadType::Amount`.
    /// The `field` value in the Rule is used to locate the numerical amount in the payload to
    /// compare to the amount stored in the rule, using the comparison operator stored in the rule.
    Amount,
    /// A rule that tells the operation finder to use the default namespace rule.
    Namespace,
}

impl ConditionType {
    /// Convert the rule to a corresponding error resulting from the rule failure.
    pub fn to_error(&self) -> ProgramError {
        match self {
            ConditionType::All | ConditionType::Any | ConditionType::Namespace => {
                RuleSetError::UnexpectedRuleSetFailure.into()
            }
            ConditionType::ProgramOwnedList => RuleSetError::ProgramOwnedListCheckFailed.into(),
            ConditionType::Amount => RuleSetError::AmountCheckFailed.into(),
        }
    }
}

impl TryFrom<u32> for ConditionType {
    // Type of the error generated.
    type Error = RuleSetError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(ConditionType::All),
            1 => Ok(ConditionType::Any),
            2 => Ok(ConditionType::ProgramOwnedList),
            3 => Ok(ConditionType::Amount),
            4 => Ok(ConditionType::Namespace),
            value => {
                panic!("invalid rule type: {}", value)
            }
        }
    }
}
