//! Errors used by the Rule Set program.
use num_derive::FromPrimitive;
use solana_program::{
    decode_error::DecodeError,
    msg,
    program_error::{PrintProgramError, ProgramError},
};
use thiserror::Error;

#[derive(Error, Clone, Debug, Eq, PartialEq, FromPrimitive)]
/// The various errors that can be returned by the Rule Set program instructions.
pub enum RuleSetError {
    /// 0 - Numerical Overflow
    #[error("Numerical Overflow")]
    NumericalOverflow,

    /// 1 - Data type mismatch
    #[error("Data type mismatch")]
    DataTypeMismatch,

    /// 2 - Incorrect account owner
    #[error("Incorrect account owner")]
    IncorrectOwner,

    /// 3 - PayloadVec Index error.
    #[error("Could not index into PayloadVec")]
    PayloadVecIndexError,

    /// 4 - Derived key invalid
    #[error("Derived key invalid")]
    DerivedKeyInvalid,

    /// 5 - Additional Signer check failed
    #[error("Additional Signer check failed")]
    AdditionalSignerCheckFailed,

    /// 6 - Pubkey Match check failed
    #[error("Pubkey Match check failed")]
    PubkeyMatchCheckFailed,

    /// 7 - Pubkey List Match check failed
    #[error("Pubkey List Match check failed")]
    PubkeyListMatchCheckFailed,

    /// 8 - Pubkey Tree Match check failed
    #[error("Pubkey Tree Match check failed")]
    PubkeyTreeMatchCheckFailed,

    /// 9 - PDA Match check failed
    #[error("PDA Match check failed")]
    PDAMatchCheckFailed,

    /// 10 - Program Owned check failed
    #[error("Program Owned check failed")]
    ProgramOwnedCheckFailed,

    /// 11 - Program Owned List check failed
    #[error("Program Owned List check failed")]
    ProgramOwnedListCheckFailed,

    /// 12 - Program Owned Tree check failed
    #[error("Program Owned Tree check failed")]
    ProgramOwnedTreeCheckFailed,

    /// 13 - Amount checked failed
    #[error("Amount checked failed")]
    AmountCheckFailed,

    /// 14 - Frequency check failed
    #[error("Frequency check failed")]
    FrequencyCheckFailed,

    /// 15 - Payer is not a signer
    #[error("Payer is not a signer")]
    PayerIsNotSigner,

    /// 16 - Feature is not implemented yet
    #[error("Not implemented")]
    NotImplemented,

    /// 17 - Borsh serialization error
    #[error("Borsh serialization error")]
    BorshSerializationError,

    /// 18 - Value in Payload or RuleSet is occupied
    #[error("Value in Payload or RuleSet is occupied")]
    ValueOccupied,

    /// 19 - Account data is empty
    #[error("Account data is empty")]
    DataIsEmpty,

    /// 20 - MessagePack deserialization error
    #[error("MessagePack deserialization error")]
    MessagePackDeserializationError,

    /// 21 - Missing account
    #[error("Missing account")]
    MissingAccount,

    /// 22 - Missing Payload value
    #[error("Missing Payload value")]
    MissingPayloadValue,

    /// 23 - RuleSet owner must be payer
    #[error("RuleSet owner must be payer")]
    RuleSetOwnerMismatch,

    /// 24 - Name too long
    #[error("Name too long")]
    NameTooLong,

    /// 25 - Name too long
    #[error("The operation retrieved is not in the selected RuleSet")]
    OperationNotFound,

    /// 26 - Rule authority is not signer
    #[error("Rule authority is not signer")]
    RuleAuthorityIsNotSigner,

    /// 27 - Unsupported RuleSet version
    #[error("Unsupported RuleSet version")]
    UnsupportedRuleSetVersion,
}

impl PrintProgramError for RuleSetError {
    fn print<E>(&self) {
        msg!(&self.to_string());
    }
}

impl From<RuleSetError> for ProgramError {
    fn from(e: RuleSetError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl<T> DecodeError<T> for RuleSetError {
    fn type_of() -> &'static str {
        "Error Thingy"
    }
}
