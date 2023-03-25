use borsh::BorshSerialize;
use solana_program::{
    msg,
    pubkey::{Pubkey, PUBKEY_BYTES},
};
use std::fmt::Display;

use crate::{
    error::RuleSetError,
    state::RuleResult,
    state_v2::{Condition, ConditionType, HEADER_SECTION},
};

pub struct Frequency<'a> {
    pub authority: &'a Pubkey,
}

impl<'a> Frequency<'a> {
    pub fn from_bytes(bytes: &'a [u8]) -> Result<Self, RuleSetError> {
        let authority = bytemuck::from_bytes::<Pubkey>(bytes);

        Ok(Self { authority })
    }

    pub fn serialize(authority: Pubkey) -> std::io::Result<Vec<u8>> {
        let length = PUBKEY_BYTES as u32;
        let mut data = Vec::with_capacity(HEADER_SECTION + length as usize);

        // Header
        // - rule type
        let rule_type = ConditionType::PubkeyMatch as u32;
        BorshSerialize::serialize(&rule_type, &mut data)?;
        // - length
        BorshSerialize::serialize(&length, &mut data)?;

        // Assert
        // - pubkey
        BorshSerialize::serialize(&authority, &mut data)?;

        Ok(data)
    }
}

impl<'a> Condition<'a> for Frequency<'a> {
    fn condition_type(&self) -> ConditionType {
        ConditionType::Frequency
    }

    fn validate(
        &self,
        _accounts: &std::collections::HashMap<
            solana_program::pubkey::Pubkey,
            &solana_program::account_info::AccountInfo,
        >,
        _payload: &crate::payload::Payload,
        _update_rule_state: bool,
        _rule_set_state_pda: &Option<&solana_program::account_info::AccountInfo>,
        rule_authority: &Option<&solana_program::account_info::AccountInfo>,
    ) -> RuleResult {
        msg!("Validating Frequency");

        if let Some(rule_authority) = rule_authority {
            // TODO: If it's the wrong account (first condition) the `IsNotASigner`
            // is misleading.  Should be improved, perhaps with a `Mismatch` error.
            if self.authority != rule_authority.key || !rule_authority.is_signer {
                return RuleResult::Error(RuleSetError::RuleAuthorityIsNotSigner.into());
            }
        } else {
            return RuleResult::Error(RuleSetError::MissingAccount.into());
        }

        RuleResult::Error(RuleSetError::NotImplemented.into())
    }
}

impl<'a> Display for Frequency<'a> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("Frequency {authority: \"")?;
        formatter.write_str(&format!("{}\"", self.authority))?;
        formatter.write_str("}")
    }
}
