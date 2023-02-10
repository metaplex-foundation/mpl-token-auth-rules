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

    /// 2 - Data slice unexpected index error
    #[error("Data slice unexpected index error")]
    DataSliceUnexpectedIndexError,

    /// 3 - Incorrect account owner
    #[error("Incorrect account owner")]
    IncorrectOwner,

    /// 4 - PayloadVec Index error.
    #[error("Could not index into PayloadVec")]
    PayloadVecIndexError,

    /// 5 - Derived key invalid
    #[error("Derived key invalid")]
    DerivedKeyInvalid,

    /// 6 - Payer is not a signer
    #[error("Payer is not a signer")]
    PayerIsNotSigner,

    /// 7 - Feature is not implemented yet
    #[error("Not implemented")]
    NotImplemented,

    /// 8 - Borsh serialization error
    #[error("Borsh serialization error")]
    BorshSerializationError,

    /// 9 - Borsh deserialization error
    #[error("Borsh deserialization error")]
    BorshDeserializationError,

    /// 10 - Value in Payload or RuleSet is occupied
    #[error("Value in Payload or RuleSet is occupied")]
    ValueOccupied,

    /// 11 - Account data is empty
    #[error("Account data is empty")]
    DataIsEmpty,

    /// 12 - MessagePack serialization error
    #[error("MessagePack serialization error")]
    MessagePackSerializationError,

    /// 13 - MessagePack deserialization error
    #[error("MessagePack deserialization error")]
    MessagePackDeserializationError,

    /// 14 - Missing account
    #[error("Missing account")]
    MissingAccount,

    /// 15 - Missing Payload value
    #[error("Missing Payload value")]
    MissingPayloadValue,

    /// 16 - RuleSet owner must be payer
    #[error("RuleSet owner must be payer")]
    RuleSetOwnerMismatch,

    /// 17 - Name too long
    #[error("Name too long")]
    NameTooLong,

    /// 18 - The operation retrieved is not in the selected RuleSet
    #[error("The operation retrieved is not in the selected RuleSet")]
    OperationNotFound,

    /// 19 - Rule authority is not signer
    #[error("Rule authority is not signer")]
    RuleAuthorityIsNotSigner,

    /// 20 - Unsupported RuleSet header version
    #[error("Unsupported RuleSet revision map version")]
    UnsupportedRuleSetRevMapVersion,

    /// 21 - Unsupported RuleSet version
    #[error("Unsupported RuleSet version")]
    UnsupportedRuleSetVersion,

    /// 22 - Unexpected RuleSet failure
    #[error("Unexpected RuleSet failure")]
    UnexpectedRuleSetFailure,

    /// 23 - RuleSet revision not available
    #[error("RuleSet revision not available")]
    RuleSetRevisionNotAvailable,

    /// 24 - Additional Signer check failed
    #[error("Additional Signer check failed")]
    AdditionalSignerCheckFailed,

    /// 25 - Pubkey Match check failed
    #[error("Pubkey Match check failed")]
    PubkeyMatchCheckFailed,

    /// 26 - Pubkey List Match check failed
    #[error("Pubkey List Match check failed")]
    PubkeyListMatchCheckFailed,

    /// 27 - Pubkey Tree Match check failed
    #[error("Pubkey Tree Match check failed")]
    PubkeyTreeMatchCheckFailed,

    /// 28 - PDA Match check failed
    #[error("PDA Match check failed")]
    PDAMatchCheckFailed,

    /// 29 - Program Owned check failed
    #[error("Program Owned check failed")]
    ProgramOwnedCheckFailed,

    /// 30 - Program Owned List check failed
    #[error("Program Owned List check failed")]
    ProgramOwnedListCheckFailed,

    /// 31 - Program Owned Tree check failed
    #[error("Program Owned Tree check failed")]
    ProgramOwnedTreeCheckFailed,

    /// 32 - Amount checked failed
    #[error("Amount checked failed")]
    AmountCheckFailed,

    /// 33 - Frequency check failed
    #[error("Frequency check failed")]
    FrequencyCheckFailed,

    /// 34 - IsWallet check failed
    #[error("IsWallet check failed")]
    IsWalletCheckFailed,

    /// 35 - Program Owned Set check failed
    #[error("Program Owned Set check failed")]
    ProgramOwnedSetCheckFailed,
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
