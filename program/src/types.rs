use std::collections::HashMap;

use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};

use crate::{error::RuleSetError, payload::Payload};

/// Max name length for any of the names used in this crate.
pub const MAX_NAME_LENGTH: usize = 32;

/// Versioning for `RuleSet` structs.
pub enum LibVersion {
    V1 = 1,
    V2,
}

impl TryFrom<u8> for LibVersion {
    type Error = RuleSetError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(LibVersion::V1),
            2 => Ok(LibVersion::V2),
            _ => Err(RuleSetError::UnsupportedRuleSetVersion),
        }
    }
}

pub trait Assertable<'a> {
    fn validate(
        &self,
        accounts: &HashMap<Pubkey, &AccountInfo>,
        payload: &Payload,
        update_rule_state: bool,
        rule_set_state_pda: &Option<&AccountInfo>,
        rule_authority: &Option<&AccountInfo>,
    ) -> ProgramResult;
}
