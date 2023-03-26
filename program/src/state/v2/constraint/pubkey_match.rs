use borsh::BorshSerialize;
use solana_program::{
    msg,
    pubkey::{Pubkey, PUBKEY_BYTES},
};

use crate::{
    error::RuleSetError,
    state::v2::{Constraint, ConstraintType, Str32, HEADER_SECTION},
    state::RuleResult,
};

/// Constraint representing a direct comparison between `Pubkey`s.
///
/// This constraint requires a `PayloadType` value of `PayloadType::Pubkey`. The `field`
/// value in the rule is used to locate the `Pubkey` in the payload to compare to the `Pubkey`
/// in the rule.
pub struct PubkeyMatch<'a> {
    /// The public key to be compared against.
    pub pubkey: &'a Pubkey,
    /// The field in the `Payload` to be compared.
    pub field: &'a Str32,
}

impl<'a> PubkeyMatch<'a> {
    /// Deserialize a constraint from a byte array.
    pub fn from_bytes(bytes: &'a [u8]) -> Result<Self, RuleSetError> {
        let (pubkey, field) = bytes.split_at(PUBKEY_BYTES);
        let pubkey = bytemuck::from_bytes::<Pubkey>(pubkey);
        let field = bytemuck::from_bytes::<Str32>(field);

        Ok(Self { pubkey, field })
    }

    /// Serialize a constraint into a byte array.
    pub fn serialize(pubkey: Pubkey, field: String) -> std::io::Result<Vec<u8>> {
        let length = (PUBKEY_BYTES + Str32::SIZE) as u32;
        let mut data = Vec::with_capacity(HEADER_SECTION + length as usize);

        // Header
        // - rule type
        let rule_type = ConstraintType::PubkeyMatch as u32;
        BorshSerialize::serialize(&rule_type, &mut data)?;
        // - length
        BorshSerialize::serialize(&length, &mut data)?;

        // Constraint
        // - pubkey
        BorshSerialize::serialize(&pubkey, &mut data)?;
        // - field
        let mut field_bytes = [0u8; Str32::SIZE];
        field_bytes[..field.len()].copy_from_slice(field.as_bytes());
        BorshSerialize::serialize(&field_bytes, &mut data)?;

        Ok(data)
    }
}

impl<'a> Constraint<'a> for PubkeyMatch<'a> {
    fn constraint_type(&self) -> ConstraintType {
        ConstraintType::PubkeyMatch
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
        msg!("Validating PubkeyMatch");

        let key = match payload.get_pubkey(&self.field.to_string()) {
            Some(pubkey) => pubkey,
            _ => return RuleResult::Error(RuleSetError::MissingPayloadValue.into()),
        };

        if key == self.pubkey {
            RuleResult::Success(self.constraint_type().to_error())
        } else {
            RuleResult::Failure(self.constraint_type().to_error())
        }
    }
}
