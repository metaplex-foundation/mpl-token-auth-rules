use solana_program::{
    msg,
    pubkey::{Pubkey, PUBKEY_BYTES},
};

use crate::{
    error::RuleSetError,
    state::{try_from_bytes, RuleResult},
    state::{
        v2::{Constraint, ConstraintType, Str32, HEADER_SECTION},
        Header,
    },
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
        let pubkey = try_from_bytes::<Pubkey>(0, PUBKEY_BYTES, bytes)?;
        let field = try_from_bytes::<Str32>(PUBKEY_BYTES, Str32::SIZE, bytes)?;

        Ok(Self { pubkey, field })
    }

    /// Serialize a constraint into a byte array.
    pub fn serialize(field: String, pubkey: Pubkey) -> Result<Vec<u8>, RuleSetError> {
        let length = (PUBKEY_BYTES + Str32::SIZE) as u32;
        let mut data = Vec::with_capacity(HEADER_SECTION + length as usize);

        // Header
        Header::serialize(ConstraintType::PubkeyMatch, length, &mut data);

        // Constraint
        // - pubkey
        data.extend(pubkey.as_ref());
        // - field
        let mut field_bytes = [0u8; Str32::SIZE];
        field_bytes[..field.len()].copy_from_slice(field.as_bytes());
        data.extend(field_bytes);

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
