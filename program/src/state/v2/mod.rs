pub mod constraint;
mod rule_set_v2;
mod rule_v2;

pub use constraint::*;
pub use rule_set_v2::*;
pub use rule_v2::*;

use bytemuck::{AnyBitPattern, NoUninit, Pod, Zeroable};
use solana_program::{account_info::AccountInfo, msg, program_error::ProgramError, pubkey::Pubkey};
use std::{collections::HashMap, fmt::Display};

use crate::{error::RuleSetError, payload::Payload, state::RuleResult, types::MAX_NAME_LENGTH};

/// Size (in bytes) of a u64 value.
pub const U64_BYTES: usize = std::mem::size_of::<u64>();

/// Re-interprets `&[u8]` as `&T`, mapping any 'PodCastError' to 'RuleSetError'.
pub(crate) fn try_from_bytes<T: AnyBitPattern>(
    start: usize,
    length: usize,
    bytes: &[u8],
) -> Result<&T, RuleSetError> {
    if start + length > bytes.len() {
        msg!(
            "Invalid range: start + length > bytes.len() ({} + {} > {})",
            start,
            length,
            bytes.len()
        );
        return Err(RuleSetError::DeserializationError);
    }

    bytemuck::try_from_bytes::<T>(&bytes[start..start + length]).map_err(|error| {
        msg!("{}", error);
        RuleSetError::DeserializationError
    })
}

/// Try to convert `&[A]` into `&[B]` (possibly with a change in length), mapping
/// 'PodCastError' to 'RuleSetError'.
pub(crate) fn try_cast_slice<A: NoUninit, B: AnyBitPattern>(
    bytes: &[A],
) -> Result<&[B], RuleSetError> {
    bytemuck::try_cast_slice(bytes).map_err(|error| {
        msg!("{}", error);
        RuleSetError::DeserializationError
    })
}

/// Struct representing a 32 byte string.
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct Str32 {
    /// The bytes of the string.
    pub value: [u8; MAX_NAME_LENGTH],
}

impl Str32 {
    /// The size of the struct in bytes.
    pub const SIZE: usize = MAX_NAME_LENGTH;
}

impl Display for Str32 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let end_index = self
            .value
            .iter()
            .position(|&x| x == b'\0')
            .unwrap_or(MAX_NAME_LENGTH);
        // return a copy of the str without any padding bytes
        let value = String::from_utf8_lossy(&self.value[..end_index]);
        formatter.write_str(&value)
    }
}

/// Struct representing a test performed by a rule.
pub trait Constraint<'a> {
    /// Validates the constraint condition.
    fn validate(
        &self,
        accounts: &HashMap<Pubkey, &AccountInfo>,
        payload: &Payload,
        update_rule_state: bool,
        rule_set_state_pda: &Option<&AccountInfo>,
        rule_authority: &Option<&AccountInfo>,
    ) -> RuleResult;

    /// Returns the type of the constraint.
    fn constraint_type(&self) -> ConstraintType;
}

#[repr(u32)]
#[derive(Clone, Copy)]
/// The struct containing every type of Rule and its associated data.
pub enum ConstraintType {
    /// Indicates that the contraint is uninitialized.
    Uninitialized,
    /// An additional signer must be present.
    AdditionalSigner,
    /// Group AND, where every rule contained must pass.
    All,
    /// Comparison against the amount of tokens being transferred.
    Amount,
    /// Group OR, where at least one rule contained must pass.
    Any,
    /// Comparison based on time between operations.
    Frequency,
    /// The true test if a pubkey can be signed from a client and therefore is a true wallet account.
    IsWallet,
    /// A rule that tells the operation finder to use the default namespace rule.
    Namespace,
    /// Negation, where the contained rule must fail.
    Not,
    /// An operation that always succeeds.
    Pass,
    /// A resulting PDA derivation of seeds must prove the account is a PDA.
    PDAMatch,
    /// The `Pubkey` must be owned by a given program.  When the `Validate` instruction is called,
    /// this rule requires a `PayloadType` value of `PayloadType::Pubkey`.
    ProgramOwned,
    /// The `Pubkey` must be owned by a program in the list of `Pubkey`s.
    ProgramOwnedList,
    /// The `Pubkey` must be owned by a member of the Merkle tree in the rule.
    ProgramOwnedTree,
    /// The comparing `Pubkey` must be in the list of `Pubkey`s.
    PubkeyListMatch,
    /// Direct comparison between `Pubkey`s.  When the `Validate` instruction is called, this rule
    /// requires a `PayloadType` value of `PayloadType::Pubkey`.
    PubkeyMatch,
    /// The comparing `Pubkey` must be a member of the Merkle tree in the rule.
    PubkeyTreeMatch,
}

impl ConstraintType {
    /// Convert the rule to a corresponding error resulting from the rule failure.
    pub fn to_error(&self) -> ProgramError {
        match self {
            ConstraintType::Uninitialized => RuleSetError::InvalidConstraintType.into(),
            ConstraintType::AdditionalSigner { .. } => {
                RuleSetError::AdditionalSignerCheckFailed.into()
            }
            ConstraintType::All
            | ConstraintType::Any
            | ConstraintType::Namespace
            | ConstraintType::Not
            | ConstraintType::Pass => RuleSetError::UnexpectedRuleSetFailure.into(),
            ConstraintType::Amount => RuleSetError::AmountCheckFailed.into(),
            ConstraintType::Frequency { .. } => RuleSetError::FrequencyCheckFailed.into(),
            ConstraintType::IsWallet { .. } => RuleSetError::IsWalletCheckFailed.into(),
            ConstraintType::PDAMatch { .. } => RuleSetError::PDAMatchCheckFailed.into(),
            ConstraintType::ProgramOwned { .. } => RuleSetError::ProgramOwnedCheckFailed.into(),
            ConstraintType::ProgramOwnedList => RuleSetError::ProgramOwnedListCheckFailed.into(),
            ConstraintType::ProgramOwnedTree { .. } => {
                RuleSetError::ProgramOwnedTreeCheckFailed.into()
            }
            ConstraintType::PubkeyListMatch { .. } => {
                RuleSetError::PubkeyListMatchCheckFailed.into()
            }
            ConstraintType::PubkeyMatch { .. } => RuleSetError::PubkeyMatchCheckFailed.into(),
            ConstraintType::PubkeyTreeMatch { .. } => {
                RuleSetError::PubkeyTreeMatchCheckFailed.into()
            }
        }
    }
}

impl TryFrom<u32> for ConstraintType {
    // Type of the error generated.
    type Error = RuleSetError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(ConstraintType::Uninitialized),
            1 => Ok(ConstraintType::AdditionalSigner),
            2 => Ok(ConstraintType::All),
            3 => Ok(ConstraintType::Amount),
            4 => Ok(ConstraintType::Any),
            5 => Ok(ConstraintType::Frequency),
            6 => Ok(ConstraintType::IsWallet),
            7 => Ok(ConstraintType::Namespace),
            8 => Ok(ConstraintType::Not),
            9 => Ok(ConstraintType::Pass),
            10 => Ok(ConstraintType::PDAMatch),
            11 => Ok(ConstraintType::ProgramOwned),
            12 => Ok(ConstraintType::ProgramOwnedList),
            13 => Ok(ConstraintType::ProgramOwnedTree),
            14 => Ok(ConstraintType::PubkeyListMatch),
            15 => Ok(ConstraintType::PubkeyMatch),
            16 => Ok(ConstraintType::PubkeyTreeMatch),
            _ => Err(RuleSetError::InvalidConstraintType),
        }
    }
}

#[repr(u64)]
#[derive(PartialEq, Eq, Debug, Clone)]
/// Operators that can be used to compare against an `Amount` rule.
pub enum Operator {
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

impl TryFrom<u64> for Operator {
    // Type of the error generated.
    type Error = RuleSetError;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Operator::Lt),
            1 => Ok(Operator::LtEq),
            2 => Ok(Operator::Eq),
            3 => Ok(Operator::GtEq),
            4 => Ok(Operator::Gt),
            _ => Err(RuleSetError::InvalidCompareOp),
        }
    }
}
