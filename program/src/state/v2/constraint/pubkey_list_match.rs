use solana_program::{
    msg,
    pubkey::{Pubkey, PUBKEY_BYTES},
};

use crate::{
    error::RuleSetError,
    state::{
        try_cast_slice,
        v2::{Constraint, ConstraintType, Str32, HEADER_SECTION},
        Header,
    },
    state::{try_from_bytes, RuleResult},
};

/// Constraint representing a test where a `Pubkey` must be in the list of `Pubkey`s.
///
/// This constraint requires a `PayloadType` value of `PayloadType::Pubkey`. The `field`
/// value in the Rule is used to locate the `Pubkey` in the payload to compare to the `Pubkey`
/// list in the rule.
pub struct PubkeyListMatch<'a> {
    /// The field in the `Payload` to be compared.
    pub field: &'a Str32,
    /// The list of public keys to be compared against.
    pub pubkeys: &'a [Pubkey],
}

impl<'a> PubkeyListMatch<'a> {
    /// Deserialize a constraint from a byte array.
    pub fn from_bytes(bytes: &'a [u8]) -> Result<Self, RuleSetError> {
        let field = try_from_bytes::<Str32>(0, Str32::SIZE, bytes)?;
        let pubkeys = try_cast_slice(&bytes[Str32::SIZE..])?;

        Ok(Self { field, pubkeys })
    }

    /// Serialize a constraint into a byte array.
    pub fn serialize(field: String, pubkeys: &[Pubkey]) -> Result<Vec<u8>, RuleSetError> {
        let length = (Str32::SIZE + (pubkeys.len() * PUBKEY_BYTES)) as u32;
        let mut data = Vec::with_capacity(HEADER_SECTION + length as usize);

        // Header
        Header::serialize(ConstraintType::PubkeyListMatch, length, &mut data);

        // Constraint
        // - field
        let mut field_bytes = [0u8; Str32::SIZE];
        field_bytes[..field.len()].copy_from_slice(field.as_bytes());
        data.extend(field_bytes);
        // - pubkeys
        pubkeys.iter().for_each(|p| {
            data.extend(p.as_ref());
        });

        Ok(data)
    }
}

impl<'a> Constraint<'a> for PubkeyListMatch<'a> {
    fn constraint_type(&self) -> ConstraintType {
        ConstraintType::PubkeyListMatch
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
        msg!("Validating PubkeyListMatch");

        let field = self.field.to_string();

        for field in field.split('|') {
            let key = match payload.get_pubkey(&field.to_string()) {
                Some(pubkey) => pubkey,
                _ => return RuleResult::Error(RuleSetError::MissingPayloadValue.into()),
            };

            if self.pubkeys.contains(key) {
                // Account owner must be in the set.
                return RuleResult::Success(self.constraint_type().to_error());
            }
        }

        RuleResult::Failure(self.constraint_type().to_error())
    }
}
