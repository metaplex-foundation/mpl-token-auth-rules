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

pub struct AdditionalSigner<'a> {
    pub account: &'a Pubkey,
}

impl<'a> AdditionalSigner<'a> {
    pub fn from_bytes(bytes: &'a [u8]) -> Result<Self, RuleSetError> {
        let account = bytemuck::from_bytes::<Pubkey>(bytes);

        Ok(Self { account })
    }

    pub fn serialize(account: Pubkey) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::with_capacity(HEADER_SECTION + PUBKEY_BYTES);

        // Header
        // - rule type
        let rule_type = ConditionType::AdditionalSigner as u32;
        BorshSerialize::serialize(&rule_type, &mut data)?;
        // - length
        let length = PUBKEY_BYTES as u32;
        BorshSerialize::serialize(&length, &mut data)?;

        // Assert
        // - rule
        BorshSerialize::serialize(&account, &mut data)?;

        Ok(data)
    }
}

impl<'a> Condition<'a> for AdditionalSigner<'a> {
    fn condition_type(&self) -> ConditionType {
        ConditionType::AdditionalSigner
    }

    fn validate(
        &self,
        accounts: &std::collections::HashMap<
            solana_program::pubkey::Pubkey,
            &solana_program::account_info::AccountInfo,
        >,
        _payload: &crate::payload::Payload,
        _update_rule_state: bool,
        _rule_set_state_pda: &Option<&solana_program::account_info::AccountInfo>,
        _rule_authority: &Option<&solana_program::account_info::AccountInfo>,
    ) -> RuleResult {
        msg!("Validating AdditionalSigner");

        if let Some(signer) = accounts.get(self.account) {
            if signer.is_signer {
                RuleResult::Success(self.condition_type().to_error())
            } else {
                RuleResult::Failure(self.condition_type().to_error())
            }
        } else {
            RuleResult::Error(RuleSetError::MissingAccount.into())
        }
    }
}

impl<'a> Display for AdditionalSigner<'a> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("AdditionalSigner {account: [")?;
        formatter.write_str(&format!("{}", self.account))?;
        formatter.write_str("]}")
    }
}
