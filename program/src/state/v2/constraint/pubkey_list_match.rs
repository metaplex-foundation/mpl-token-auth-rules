use std::fmt::Display;

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
        let (field, pubkeys) = bytes.split_at(Str32::SIZE);
        let field = bytemuck::from_bytes::<Str32>(field);
        let pubkeys = bytemuck::cast_slice(pubkeys);

        Ok(Self { field, pubkeys })
    }

    /// Serialize a constraint into a byte array.
    pub fn serialize(field: String, pubkeys: &[Pubkey]) -> std::io::Result<Vec<u8>> {
        let length = (Str32::SIZE + (pubkeys.len() * PUBKEY_BYTES)) as u32;
        let mut data = Vec::with_capacity(HEADER_SECTION + length as usize);

        // Header
        // - rule type
        let rule_type = ConstraintType::PubkeyListMatch as u32;
        BorshSerialize::serialize(&rule_type, &mut data)?;
        // - length
        BorshSerialize::serialize(&length, &mut data)?;

        // Constraint
        // - field
        let mut field_bytes = [0u8; Str32::SIZE];
        field_bytes[..field.len()].copy_from_slice(field.as_bytes());
        BorshSerialize::serialize(&field_bytes, &mut data)?;
        // - programs
        pubkeys.iter().for_each(|x| {
            BorshSerialize::serialize(x, &mut data).unwrap();
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

    /// Return a string representation of the constraint.
    fn to_text(&self, indent: usize) -> String {
        let mut output = String::new();
        output.push_str(&format!("{:1$}!", "PubkeyListMatch {\n", indent));
        output.push_str(&format!("{:1$}!", "pubkeys: [", indent * 2));

        for (i, p) in self.pubkeys.iter().enumerate() {
            output.push_str(&format!(
                "\"{:2$}\"{}",
                p,
                if i > 0 { ", " } else { "" },
                indent * 3
            ));
        }

        output.push_str(&format!("{:1$}!,", "]", indent * 2));
        output.push_str(&format!(
            "{:1$}!",
            &format!("field: \"{}\"\n", self.field),
            indent * 2
        ));
        output.push_str(&format!("{:1$}!", "}", indent));
        output
    }
}

impl<'a> Display for PubkeyListMatch<'a> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.to_text(0))
    }
}
