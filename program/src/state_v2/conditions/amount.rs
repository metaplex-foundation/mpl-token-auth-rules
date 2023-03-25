use borsh::BorshSerialize;
use solana_program::msg;
use std::fmt::Display;

use crate::{
    error::RuleSetError,
    state::RuleResult,
    state_v2::{CompareOp, Condition, ConditionType, Str32, HEADER_SECTION, U64_BYTES},
};

pub struct Amount<'a> {
    pub amount: &'a u64,
    pub operator: &'a u64,
    pub field: &'a Str32,
}

impl<'a> Amount<'a> {
    pub fn from_bytes(bytes: &'a [u8]) -> Result<Self, RuleSetError> {
        // amount
        let amount = bytemuck::from_bytes::<u64>(&bytes[..U64_BYTES]);
        let mut cursor = U64_BYTES;

        // operator
        let operator = bytemuck::from_bytes::<u64>(&bytes[cursor..cursor + U64_BYTES]);
        cursor += U64_BYTES;

        // field
        let field = bytemuck::from_bytes::<Str32>(&bytes[cursor..cursor + Str32::SIZE]);

        Ok(Self {
            amount,
            operator,
            field,
        })
    }

    pub fn serialize(amount: u64, operator: CompareOp, field: String) -> std::io::Result<Vec<u8>> {
        // length of the assert
        let length = (U64_BYTES + U64_BYTES + Str32::SIZE) as u32;
        let mut data = Vec::with_capacity(HEADER_SECTION + length as usize);

        // Header
        // - rule type
        let rule_type = ConditionType::Amount as u32;
        BorshSerialize::serialize(&rule_type, &mut data)?;
        // - length
        BorshSerialize::serialize(&length, &mut data)?;

        // Assert
        // - amount
        BorshSerialize::serialize(&amount, &mut data)?;
        // - operator
        let operator = operator as u64;
        BorshSerialize::serialize(&operator, &mut data)?;
        // - field
        let mut field_bytes = [0u8; Str32::SIZE];
        field_bytes[..field.len()].copy_from_slice(field.as_bytes());
        BorshSerialize::serialize(&field_bytes, &mut data)?;

        Ok(data)
    }
}

impl<'a> Condition<'a> for Amount<'a> {
    fn condition_type(&self) -> ConditionType {
        ConditionType::Amount
    }

    fn validate(
        &self,
        _accounts: &std::collections::HashMap<
            solana_program::pubkey::Pubkey,
            &solana_program::account_info::AccountInfo,
        >,
        payload: &crate::payload::Payload,
        _update_rule_state: bool,
        _rule_set_state_pda: &Option<&solana_program::account_info::AccountInfo>,
        _rule_authority: &Option<&solana_program::account_info::AccountInfo>,
    ) -> RuleResult {
        msg!("Validating Amount");
        let condition_type = self.condition_type();

        if let Some(payload_amount) = &payload.get_amount(&self.field.to_string()) {
            let operator_fn = match CompareOp::try_from(*self.operator) {
                Ok(CompareOp::Lt) => PartialOrd::lt,
                Ok(CompareOp::LtEq) => PartialOrd::le,
                Ok(CompareOp::Eq) => PartialEq::eq,
                Ok(CompareOp::Gt) => PartialOrd::gt,
                Ok(CompareOp::GtEq) => PartialOrd::ge,
                // sanity check: the value is checked at creation
                Err(_) => return RuleResult::Failure(condition_type.to_error()),
            };

            if operator_fn(payload_amount, self.amount) {
                RuleResult::Success(condition_type.to_error())
            } else {
                RuleResult::Failure(condition_type.to_error())
            }
        } else {
            RuleResult::Error(RuleSetError::MissingPayloadValue.into())
        }
    }
}

impl<'a> Display for Amount<'a> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("Amount {")?;
        formatter.write_str(&format!("amount: {}, ", self.amount))?;
        formatter.write_str(&format!("operator: {}, ", self.operator))?;
        formatter.write_str(&format!("field: \"{}\"", self.field))?;
        formatter.write_str("}")
    }
}
