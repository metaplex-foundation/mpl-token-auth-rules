use num_derive::FromPrimitive;
use solana_program::{
    decode_error::DecodeError,
    msg,
    program_error::{PrintProgramError, ProgramError},
};
use thiserror::Error;

#[derive(Error, Clone, Debug, Eq, PartialEq, FromPrimitive)]
pub enum RuleSetError {
    /// 2 - Error description
    #[error("Error message")]
    ErrorName,

    /// 1 - Numerical Overflow
    #[error("Numerical Overflow")]
    NumericalOverflow,

    /// 2 - Data type mismatch
    #[error("Data type mismatch")]
    DataTypeMismatch,

    /// 3 - Incorrect account owner
    #[error("Incorrect account owner")]
    IncorrectOwner,

    /// 4 - PayloadVec Index error.
    #[error("Could not index into PayloadVec")]
    PayloadVecIndexError,
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
