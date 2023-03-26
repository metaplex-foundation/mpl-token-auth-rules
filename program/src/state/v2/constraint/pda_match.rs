use borsh::BorshSerialize;
use solana_program::{
    msg,
    pubkey::{Pubkey, PUBKEY_BYTES},
};

use crate::{
    error::RuleSetError,
    state::v2::{Constraint, ConstraintType, Str32, HEADER_SECTION},
    state::RuleResult,
    utils::assert_derivation,
};

const DEFAULT_PUBKEY: Pubkey = Pubkey::new_from_array([0u8; 32]);

/// Constraint representing a test where a resulting PDA derivation of seeds must prove the
/// account is a PDA.
///
/// This constraint requires `PayloadType` values of `PayloadType::Seeds`. The `field` values
/// in the Rule are used to locate them in the `Payload`.  The seeds in the `Payload` and the
/// program ID stored in the Rule are used to derive the PDA from the `Payload`.
pub struct PDAMatch<'a> {
    /// The program used for the PDA derivation. If a zeroed (default) pubkey is used then
    /// the account owner is used.
    pub program: &'a Pubkey,
    /// The field in the `Payload` to be compared when looking for the PDA.
    pub pda_field: &'a Str32,
    /// The field in the `Payload` to be compared when looking for the seeds.
    pub seeds_field: &'a Str32,
}

impl<'a> PDAMatch<'a> {
    /// Deserialize a constraint from a byte array.
    pub fn from_bytes(bytes: &'a [u8]) -> Result<Self, RuleSetError> {
        let program = bytemuck::from_bytes::<Pubkey>(&bytes[..PUBKEY_BYTES]);
        let mut cursor = PUBKEY_BYTES;

        let pda_field = bytemuck::from_bytes::<Str32>(&bytes[cursor..cursor + Str32::SIZE]);
        cursor += Str32::SIZE;

        let seeds_field = bytemuck::from_bytes::<Str32>(&bytes[cursor..cursor + Str32::SIZE]);

        Ok(Self {
            program,
            pda_field,
            seeds_field,
        })
    }

    /// Serialize a constraint into a byte array.
    pub fn serialize(
        program: Pubkey,
        pda_field: String,
        seeds_field: String,
    ) -> std::io::Result<Vec<u8>> {
        let length = (PUBKEY_BYTES + Str32::SIZE + Str32::SIZE) as u32;
        let mut data = Vec::with_capacity(HEADER_SECTION + length as usize);

        // Header
        // - rule type
        let rule_type = ConstraintType::PDAMatch as u32;
        BorshSerialize::serialize(&rule_type, &mut data)?;
        // - length
        BorshSerialize::serialize(&length, &mut data)?;

        // Constraint
        // - program
        BorshSerialize::serialize(&program, &mut data)?;
        // - pda_field
        let mut field_bytes = [0u8; Str32::SIZE];
        field_bytes[..pda_field.len()].copy_from_slice(pda_field.as_bytes());
        BorshSerialize::serialize(&field_bytes, &mut data)?;
        // - seeds_field
        let mut field_bytes = [0u8; Str32::SIZE];
        field_bytes[..seeds_field.len()].copy_from_slice(seeds_field.as_bytes());
        BorshSerialize::serialize(&field_bytes, &mut data)?;

        Ok(data)
    }
}

impl<'a> Constraint<'a> for PDAMatch<'a> {
    fn constraint_type(&self) -> ConstraintType {
        ConstraintType::PDAMatch
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
        msg!("Validating PDAMatch");

        // Get the PDA from the payload.
        let account = match payload.get_pubkey(&self.pda_field.to_string()) {
            Some(pubkey) => pubkey,
            _ => return RuleResult::Error(RuleSetError::MissingPayloadValue.into()),
        };

        // Get the derivation seeds from the payload.
        let seeds = match payload.get_seeds(&self.seeds_field.to_string()) {
            Some(seeds) => seeds,
            _ => return RuleResult::Error(RuleSetError::MissingPayloadValue.into()),
        };

        // Get the program ID to use for the PDA derivation from the Rule.
        let program = match self.program {
            &DEFAULT_PUBKEY => {
                // If the Pubkey is the default, then assume the program ID is the account owner.
                match accounts.get(account) {
                    Some(account) => account.owner,
                    _ => return RuleResult::Error(RuleSetError::MissingAccount.into()),
                }
            }
            // If the Pubkey is stored in the rule, use that value.
            _ => self.program,
        };

        // Convert the Vec of Vec into Vec of u8 slices.
        let vec_of_slices = seeds
            .seeds
            .iter()
            .map(Vec::as_slice)
            .collect::<Vec<&[u8]>>();

        if let Ok(_bump) = assert_derivation(program, account, &vec_of_slices) {
            RuleResult::Success(self.constraint_type().to_error())
        } else {
            RuleResult::Failure(self.constraint_type().to_error())
        }
    }
}
