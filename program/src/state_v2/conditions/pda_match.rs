use borsh::BorshSerialize;
use solana_program::{
    msg,
    pubkey::{Pubkey, PUBKEY_BYTES},
};
use std::fmt::Display;

use crate::{
    error::RuleSetError,
    state::RuleResult,
    state_v2::{Condition, ConditionType, Str32, HEADER_SECTION},
    utils::assert_derivation,
};

const DEFAULT_PUBKEY: Pubkey = Pubkey::new_from_array([0u8; 32]);

pub struct PDAMatch<'a> {
    pub program: &'a Pubkey,
    pub pda_field: &'a Str32,
    pub seeds_field: &'a Str32,
}

impl<'a> PDAMatch<'a> {
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

    pub fn serialize(
        program: Pubkey,
        pda_field: String,
        seeds_field: String,
    ) -> std::io::Result<Vec<u8>> {
        let length = (PUBKEY_BYTES + Str32::SIZE + Str32::SIZE) as u32;
        let mut data = Vec::with_capacity(HEADER_SECTION + length as usize);

        // Header
        // - rule type
        let rule_type = ConditionType::PDAMatch as u32;
        BorshSerialize::serialize(&rule_type, &mut data)?;
        // - length
        BorshSerialize::serialize(&length, &mut data)?;

        // Assert
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

impl<'a> Condition<'a> for PDAMatch<'a> {
    fn condition_type(&self) -> ConditionType {
        ConditionType::PDAMatch
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
            RuleResult::Success(self.condition_type().to_error())
        } else {
            RuleResult::Failure(self.condition_type().to_error())
        }
    }
}

impl<'a> Display for PDAMatch<'a> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("PDAMatch {program: ")?;
        formatter.write_str(&format!("\"{}\",", self.program))?;
        formatter.write_str(&format!(" pda_field: \"{}\",", self.pda_field))?;
        formatter.write_str(&format!(" seeds_field: \"{}\"", self.seeds_field))?;
        formatter.write_str(" }")
    }
}
