use std::fmt::Display;

use borsh::BorshSerialize;
use solana_program::{msg, pubkey::PUBKEY_BYTES};

use crate::{
    error::RuleSetError,
    state::v2::{Constraint, ConstraintType, Str32, HEADER_SECTION},
    state::RuleResult,
    utils::compute_merkle_root,
};

/// Constraing representing a test where a `Pubkey` must be a member of the Merkle tree in the rule.
///
/// This constraint requires `PayloadType` values of `PayloadType::Pubkey` and `PayloadType::MerkleProof`.
/// The `field` values in the Rule are used to locate them in the `Payload`. The `Pubkey` and the proof
/// are used to calculate a Merkle root which is compared against the root stored in the rule.
pub struct PubkeyTreeMatch<'a> {
    /// The field in the `Payload` to be compared when looking for the `Pubkey`.
    pub pubkey_field: &'a Str32,
    /// The field in the `Payload` to be compared when looking for the Merkle proof.
    pub proof_field: &'a Str32,
    /// The root of the Merkle tree.
    pub root: &'a [u8; PUBKEY_BYTES],
}

impl<'a> PubkeyTreeMatch<'a> {
    /// Deserialize a constraint from a byte array.
    pub fn from_bytes(bytes: &'a [u8]) -> Result<Self, RuleSetError> {
        let pubkey_field = bytemuck::from_bytes::<Str32>(&bytes[..Str32::SIZE]);
        let mut cursor = Str32::SIZE;

        let proof_field = bytemuck::from_bytes::<Str32>(&bytes[cursor..cursor + Str32::SIZE]);
        cursor += Str32::SIZE;

        let root = bytemuck::from_bytes::<[u8; 32]>(&bytes[cursor..]);

        Ok(Self {
            pubkey_field,
            proof_field,
            root,
        })
    }

    /// Serialize a constraint into a byte array.
    pub fn serialize(
        pubkey_field: String,
        proof_field: String,
        root: &[u8; PUBKEY_BYTES],
    ) -> std::io::Result<Vec<u8>> {
        let length = (Str32::SIZE + Str32::SIZE + PUBKEY_BYTES) as u32;
        let mut data = Vec::with_capacity(HEADER_SECTION + length as usize);

        // Header
        // - rule type
        let rule_type = ConstraintType::PubkeyTreeMatch as u32;
        BorshSerialize::serialize(&rule_type, &mut data)?;
        // - length
        BorshSerialize::serialize(&length, &mut data)?;

        // Constraint
        // - pubkey_field
        let mut field_bytes = [0u8; Str32::SIZE];
        field_bytes[..pubkey_field.len()].copy_from_slice(pubkey_field.as_bytes());
        BorshSerialize::serialize(&field_bytes, &mut data)?;
        // - pubkey_field
        let mut field_bytes = [0u8; Str32::SIZE];
        field_bytes[..proof_field.len()].copy_from_slice(proof_field.as_bytes());
        BorshSerialize::serialize(&field_bytes, &mut data)?;
        // - root
        data.extend_from_slice(root);

        Ok(data)
    }
}

impl<'a> Constraint<'a> for PubkeyTreeMatch<'a> {
    fn constraint_type(&self) -> ConstraintType {
        ConstraintType::PubkeyTreeMatch
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
        msg!("Validating PubkeyTreeMatch");

        // Get the `Pubkey` we are checking from the payload.
        let leaf = match payload.get_pubkey(&self.pubkey_field.to_string()) {
            Some(pubkey) => pubkey,
            _ => return RuleResult::Error(RuleSetError::MissingPayloadValue.into()),
        };

        // Get the Merkle proof from the payload.
        let merkle_proof = match payload.get_merkle_proof(&self.proof_field.to_string()) {
            Some(merkle_proof) => merkle_proof,
            _ => return RuleResult::Error(RuleSetError::MissingPayloadValue.into()),
        };

        // Check if the computed hash (root) is equal to the root in the rule.
        let computed_root = compute_merkle_root(leaf, merkle_proof);

        if computed_root == *self.root {
            RuleResult::Success(self.constraint_type().to_error())
        } else {
            RuleResult::Failure(self.constraint_type().to_error())
        }
    }

    /// Return a string representation of the constraint.
    fn to_text(&self, indent: usize) -> String {
        let mut output = String::new();
        output.push_str(&format!("{:1$}!", "PubkeyTreeMatch {\n", indent));
        output.push_str(&format!(
            "{:1$}!",
            &format!("pubkey_field: \"{}\",\n", self.pubkey_field),
            indent * 2
        ));
        output.push_str(&format!(
            "{:1$}!",
            &format!("proof_field: \"{}\",\n", self.proof_field),
            indent * 2
        ));
        output.push_str(&format!(
            "{:1$}!",
            &format!("root: \"{:?}\"\n", self.root),
            indent * 2
        ));
        output.push_str(&format!("{:1$}!", "}", indent));
        output
    }
}

impl<'a> Display for PubkeyTreeMatch<'a> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("PubkeyTreeMatch {")?;
        formatter.write_str(&format!("pubkey_field: \"{}\",", self.pubkey_field))?;
        formatter.write_str(&format!("proof_field: \"{}\",", self.proof_field))?;
        formatter.write_str(&format!("root: {:?}", self.root))?;
        formatter.write_str("}")
    }
}
