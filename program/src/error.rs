use num_derive::FromPrimitive;
use solana_program::{
    decode_error::DecodeError,
    msg,
    program_error::{PrintProgramError, ProgramError},
};
use thiserror::Error;

#[derive(Error, Clone, Debug, Eq, PartialEq, FromPrimitive)]
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

    /// 7 - Derived Key Match check failed
    #[error("Derived Key Match check failed")]
    DerivedKeyMatchCheckFailed,

    /// 8 - Program Owned check failed
    #[error("Program Owned check failed")]
    ProgramOwnedCheckFailed,

    /// 9 - Amount checked failed
    #[error("Amount checked failed")]
    AmountCheckFailed,

    /// 10 - Frequency check failed
    #[error("Frequency check failed")]
    FrequencyCheckFailed,

    /// 11 - Pubkey Tree Match check failed
    #[error("Pubkey Tree Match check failed")]
    PubkeyTreeMatchCheckFailed,

    /// 12 - Payer is not a signer
    #[error("Payer is not a signer")]
    PayerIsNotSigner,

    /// 13 - Feature is not implemented yet
    #[error("Not implemented")]
    NotImplemented,

    /// 14 - Borsh Serialization Error
    #[error("Borsh Serialization Error")]
    BorshSerializationError,
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
