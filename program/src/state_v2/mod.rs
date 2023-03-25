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

pub trait Condition<'a>: Display {
    fn validate(
        &self,
        _accounts: &HashMap<Pubkey, &AccountInfo>,
        _payload: &Payload,
        _update_rule_state: bool,
        _rule_set_state_pda: &Option<&AccountInfo>,
        _rule_authority: &Option<&AccountInfo>,
    ) -> RuleResult;

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
            _ => Err(RuleSetError::InvalidCompareOp),
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
    /// Negation, where the contained rule must fail.
    Not,
    /// An additional signer must be present.  When the `Validate` instruction is called, this rule
    /// does not require any `Payload` values, but the additional signer account must be provided
    /// to `Validate` via the `additional_rule_accounts` argument so that whether it is a signer
    /// can be retrieved from its `AccountInfo` struct.
    AdditionalSigner,
    /// Direct comparison between `Pubkey`s.  When the `Validate` instruction is called, this rule
    /// requires a `PayloadType` value of `PayloadType::Pubkey`.  The `field` value in the rule is
    /// used to locate the `Pubkey` in the payload to compare to the `Pubkey` in the rule.
    PubkeyMatch,
    /// The comparing `Pubkey` must be in the list of `Pubkey`s.  When the `Validate` instruction
    /// is called, this rule requires a `PayloadType` value of `PayloadType::Pubkey`.  The `field`
    /// value in the Rule is used to locate the `Pubkey` in the payload to compare to the `Pubkey`
    /// list in the rule.
    PubkeyListMatch,
    /// The comparing `Pubkey` must be a member of the Merkle tree in the rule.  When the
    /// `Validate` instruction is called, this rule requires `PayloadType` values of
    /// `PayloadType::Pubkey` and `PayloadType::MerkleProof`.  The `field` values in the Rule are
    /// used to locate them in the `Payload`.  The `Pubkey` and the proof are used to calculate
    /// a Merkle root which is compared against the root stored in the rule.
    PubkeyTreeMatch,
    /// A resulting PDA derivation of seeds must prove the account is a PDA.  When the `Validate`
    /// instruction is called, this rule requires `PayloadType` values of `PayloadType::Seeds`.
    /// The `field` values in the Rule are used to locate them in the `Payload`.  The seeds in the
    /// `Payload` and the program ID stored in the Rule are used to derive the PDA from the
    /// `Payload`.
    PDAMatch,
    /// The `Pubkey` must be owned by a given program.  When the `Validate` instruction is called,
    /// this rule requires a `PayloadType` value of `PayloadType::Pubkey`.  The `field` value in
    /// the rule is used to locate the `Pubkey` in the payload for which the owner must be the
    /// program in the rule.  Note this same `Pubkey` account must also be provided to `Validate`
    /// via the `additional_rule_accounts` argument.  This is so that the `Pubkey`'s owner can be
    /// found from its `AccountInfo` struct.
    ProgramOwned,
    /// The `Pubkey` must be owned by a member of the Merkle tree in the rule.  When the `Validate`
    /// instruction is called, this rule requires `PayloadType` values of `PayloadType::Pubkey` and
    /// `PayloadType::MerkleProof`.  The `field` values in the Rule are used to locate them in the
    /// `Payload`.  Note this same `Pubkey` account must also be provided to `Validate` via the
    /// `additional_rule_accounts` argument.  This is so that the `Pubkey`'s owner can be found
    /// from its `AccountInfo` struct.  The owner and the proof are then used to calculate a Merkle
    /// root, which is compared against the root stored in the rule.
    ProgramOwnedTree,
    /// Comparison based on time between operations.  Currently not implemented.  This rule
    /// is planned check to ensure a certain amount of time has passed.  This rule will make use
    /// of the `rule_set_state_pda` optional account passed into `Validate`, and will require
    /// the optional `rule_authority` account to sign.
    Frequency,
    /// The true test if a pubkey can be signed from a client and therefore is a true wallet account.
    /// The details of this rule are as follows: a wallet is defined as being both owned by the
    /// System Program and the address is on-curve.  The `field` value in the rule is used to
    /// locate the `Pubkey` in the payload that must be on-curve and for which the owner must be
    /// the System Program.  Note this same `Pubkey` account must also be provided to `Validate`
    /// via the `additional_rule_accounts` argument.  This is so that the `Pubkey`'s owner can be
    /// found from its `AccountInfo` struct.
    IsWallet,
    /// An operation that always succeeds.
    Pass,
}

impl ConditionType {
    /// Convert the rule to a corresponding error resulting from the rule failure.
    pub fn to_error(&self) -> ProgramError {
        match self {
            ConditionType::All
            | ConditionType::Any
            | ConditionType::Pass
            | ConditionType::Namespace
            | ConditionType::Not => RuleSetError::UnexpectedRuleSetFailure.into(),
            ConditionType::ProgramOwnedList => RuleSetError::ProgramOwnedListCheckFailed.into(),
            ConditionType::Amount => RuleSetError::AmountCheckFailed.into(),
            ConditionType::AdditionalSigner { .. } => {
                RuleSetError::AdditionalSignerCheckFailed.into()
            }
            ConditionType::PubkeyMatch { .. } => RuleSetError::PubkeyMatchCheckFailed.into(),
            ConditionType::PubkeyListMatch { .. } => {
                RuleSetError::PubkeyListMatchCheckFailed.into()
            }
            ConditionType::PubkeyTreeMatch { .. } => {
                RuleSetError::PubkeyTreeMatchCheckFailed.into()
            }
            ConditionType::PDAMatch { .. } => RuleSetError::PDAMatchCheckFailed.into(),
            ConditionType::ProgramOwned { .. } => RuleSetError::ProgramOwnedCheckFailed.into(),
            ConditionType::ProgramOwnedTree { .. } => {
                RuleSetError::ProgramOwnedTreeCheckFailed.into()
            }
            ConditionType::Frequency { .. } => RuleSetError::FrequencyCheckFailed.into(),
            ConditionType::IsWallet { .. } => RuleSetError::IsWalletCheckFailed.into(),
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
