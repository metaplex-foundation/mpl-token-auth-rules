use std::fmt::Display;

use borsh::BorshSerialize;
use solana_program::{msg, program_error::ProgramError, pubkey::PUBKEY_BYTES};

use crate::{
    error::RuleSetError,
    state::RuleResult,
    state_v2::{Condition, ConditionType, Str32, HEADER_SECTION},
    utils::{compute_merkle_root, is_zeroed},
};

pub struct ProgramOwnedTree<'a> {
    pub pubkey_field: &'a Str32,
    pub proof_field: &'a Str32,
    pub root: &'a [u8; PUBKEY_BYTES],
}

impl<'a> ProgramOwnedTree<'a> {
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

    pub fn serialize(
        pubkey_field: String,
        proof_field: String,
        root: &[u8; PUBKEY_BYTES],
    ) -> std::io::Result<Vec<u8>> {
        let length = (Str32::SIZE + Str32::SIZE + PUBKEY_BYTES) as u32;
        let mut data = Vec::with_capacity(HEADER_SECTION + length as usize);

        // Header
        // - rule type
        let rule_type = ConditionType::PubkeyTreeMatch as u32;
        BorshSerialize::serialize(&rule_type, &mut data)?;
        // - length
        BorshSerialize::serialize(&length, &mut data)?;

        // Assert
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

impl<'a> Condition<'a> for ProgramOwnedTree<'a> {
    fn condition_type(&self) -> ConditionType {
        ConditionType::PubkeyTreeMatch
    }

    fn validate(
        &self,
        accounts: &std::collections::HashMap<
            solana_program::pubkey::Pubkey,
            &solana_program::account_info::AccountInfo,
        >,
        payload: &crate::payload::Payload,
        _update_rule_state: bool,
        _rule_set_state_pda: &Option<&solana_program::account_info::AccountInfo>,
        _rule_authority: &Option<&solana_program::account_info::AccountInfo>,
    ) -> RuleResult {
        msg!("Validating ProgramOwnedTree");

        // Get the `Pubkey` we are checking from the payload.
        let key = match payload.get_pubkey(&self.pubkey_field.to_string()) {
            Some(pubkey) => pubkey,
            _ => return RuleResult::Error(RuleSetError::MissingPayloadValue.into()),
        };

        // Get the `AccountInfo` struct for the `Pubkey`.
        let account = match accounts.get(key) {
            Some(account) => account,
            _ => return RuleResult::Error(RuleSetError::MissingAccount.into()),
        };

        let data = match account.data.try_borrow() {
            Ok(data) => data,
            Err(_) => return RuleResult::Error(ProgramError::AccountBorrowFailed),
        };

        // Account must have nonzero data to count as program-owned.
        if is_zeroed(&data) {
            // Print helpful errors.
            if data.len() == 0 {
                msg!("Account data is empty");
            } else {
                msg!("Account data is zeroed");
            }

            return RuleResult::Error(RuleSetError::DataIsEmpty.into());
        }

        // The account owner is the leaf.
        let leaf = account.owner;

        // Get the Merkle proof from the payload.
        let merkle_proof = match payload.get_merkle_proof(&self.proof_field.to_string()) {
            Some(merkle_proof) => merkle_proof,
            _ => return RuleResult::Error(RuleSetError::MissingPayloadValue.into()),
        };

        // Check if the computed hash (root) is equal to the root in the rule.
        let computed_root = compute_merkle_root(leaf, merkle_proof);
        if computed_root == *self.root {
            RuleResult::Success(self.condition_type().to_error())
        } else {
            RuleResult::Failure(self.condition_type().to_error())
        }
    }
}

impl<'a> Display for ProgramOwnedTree<'a> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("ProgramOwnedTree {")?;
        formatter.write_str(&format!("pubkey_field: \"{}\",", self.pubkey_field))?;
        formatter.write_str(&format!("proof_field: \"{}\",", self.proof_field))?;
        formatter.write_str(&format!("root: {:?}", self.root))?;
        formatter.write_str("}")
    }
}
