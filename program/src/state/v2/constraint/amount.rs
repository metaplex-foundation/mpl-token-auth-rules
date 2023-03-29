use solana_program::msg;

use crate::{
    error::RuleSetError,
    state::{try_from_bytes, RuleResult},
    state::{
        v2::{Constraint, ConstraintType, Operator, Str32, HEADER_SECTION, U64_BYTES},
        Header,
    },
};

/// Constraint representing a comparison against the amount of tokens being transferred.
///
/// This constraint requires a `PayloadType` value of `PayloadType::Amount`. The `field`
/// value in the Rule is used to locate the numerical amount in the payload to compare to
/// the amount stored in the rule, using the comparison operator stored in the rule.
pub struct Amount<'a> {
    /// The amount to be compared against.
    pub amount: &'a u64,
    /// The operator to be used in the comparison.
    pub operator: &'a u64,
    /// The field the amount is stored in.
    pub field: &'a Str32,
}

impl<'a> Amount<'a> {
    /// Deserialize a constraint from a byte array.
    pub fn from_bytes(bytes: &'a [u8]) -> Result<Self, RuleSetError> {
        // amount
        let amount = try_from_bytes::<u64>(0, U64_BYTES, bytes)?;
        let mut cursor = U64_BYTES;

        // operator
        let operator = try_from_bytes::<u64>(cursor, U64_BYTES, bytes)?;
        cursor += U64_BYTES;

        // field
        let field = try_from_bytes::<Str32>(cursor, Str32::SIZE, bytes)?;

        Ok(Self {
            amount,
            operator,
            field,
        })
    }

    /// Serialize a constraint into a byte array.
    pub fn serialize(amount: u64, operator: Operator, field: String) -> std::io::Result<Vec<u8>> {
        // length of the assert
        let length = (U64_BYTES + U64_BYTES + Str32::SIZE) as u32;
        let mut data = Vec::with_capacity(HEADER_SECTION + length as usize);

        // Header
        Header::serialize(ConstraintType::Amount, length, &mut data);

        // Constraint
        // - amount
        data.extend(u64::to_le_bytes(amount));
        // - operator
        data.extend(u64::to_le_bytes(operator as u64));
        // - field
        let mut field_bytes = [0u8; Str32::SIZE];
        field_bytes[..field.len()].copy_from_slice(field.as_bytes());
        data.extend(field_bytes);

        Ok(data)
    }
}

impl<'a> Constraint<'a> for Amount<'a> {
    fn constraint_type(&self) -> ConstraintType {
        ConstraintType::Amount
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
        let condition_type = self.constraint_type();

        if let Some(payload_amount) = &payload.get_amount(&self.field.to_string()) {
            let operator_fn = match Operator::try_from(*self.operator) {
                Ok(Operator::Lt) => PartialOrd::lt,
                Ok(Operator::LtEq) => PartialOrd::le,
                Ok(Operator::Eq) => PartialEq::eq,
                Ok(Operator::Gt) => PartialOrd::gt,
                Ok(Operator::GtEq) => PartialOrd::ge,
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
