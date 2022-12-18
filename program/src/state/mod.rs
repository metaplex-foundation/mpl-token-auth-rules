use borsh::{BorshDeserialize, BorshSerialize};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
};

mod frequency;
mod rule_set;
mod rules;

pub use frequency::*;
pub use rule_set::*;
pub use rules::*;

use crate::{error::RuleSetError, utils::assert_owned_by};

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone, Copy, FromPrimitive)]
pub enum Key {
    Uninitialized,
    Frequency,
}

pub trait SolanaAccount: BorshSerialize + BorshDeserialize {
    /// Get the `Key` for this `Account`.  This key is to be stored in the first byte of the
    /// `Account` data.
    fn key() -> Key;

    /// Get the size of the `Account` data.
    fn size() -> usize;

    /// BorshDeserialize the `AccountInfo` into the Rust data structure.
    fn from_account_info(account: &AccountInfo) -> Result<Self, ProgramError> {
        let data = account
            .data
            .try_borrow()
            .map_err(|_| ProgramError::AccountBorrowFailed)?;

        if !Self::is_correct_account_type_and_size(&data, Self::key(), Self::size()) {
            return Err(RuleSetError::DataTypeMismatch.into());
        }

        let data = Self::try_from_slice(&data)?;

        // Check that this account is owned by this program.
        assert_owned_by(account, &crate::ID)?;

        Ok(data)
    }

    /// BorshSerialize the Rust data structure into the `Account` data.
    fn to_account_data(&self, account: &AccountInfo) -> ProgramResult {
        let mut data = account.try_borrow_mut_data()?;
        self.serialize(&mut *data).map_err(Into::into)
    }
}

trait PrivateSolanaAccountMethods: SolanaAccount {
    const KEY_BYTE: usize = 0;

    // Check the `Key` byte and the data size to determine if this data represents the correct
    // account types.
    fn is_correct_account_type_and_size(data: &[u8], data_type: Key, data_size: usize) -> bool {
        let key: Option<Key> = Key::from_u8(data[Self::KEY_BYTE]);
        match key {
            Some(key) => key == data_type && data.len() == data_size,
            None => false,
        }
    }
}

impl<T: SolanaAccount> PrivateSolanaAccountMethods for T {}
