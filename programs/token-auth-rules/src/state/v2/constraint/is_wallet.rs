use solana_program::{
    msg, system_program,
    sysvar::{self, instructions::get_instruction_relative},
};

use crate::{
    error::RuleSetError,
    state::{try_from_bytes, RuleResult},
    state::{
        v2::{Constraint, ConstraintType, Str32, HEADER_SECTION},
        Header,
    },
    utils::cmp_pubkeys,
};

/// Constraint that represents a test on whether a pubkey can be signed from a client and therefore
/// is a true wallet account or not.
///
/// The details of this constraint are as follows: a wallet is defined as being both owned by the
/// System Program and the address is on-curve.  The `field` value in the rule is used to
/// locate the `Pubkey` in the payload that must be on-curve and for which the owner must be
/// the System Program.  Note this same `Pubkey` account must also be provided to `Validate`
/// via the `additional_rule_accounts` argument.  This is so that the `Pubkey`'s owner can be
/// found from its `AccountInfo` struct.
pub struct IsWallet<'a> {
    /// The field in the `Payload` to be checked.
    pub field: &'a Str32,
}

impl<'a> IsWallet<'a> {
    /// Deserialize a constraint from a byte array.
    pub fn from_bytes(bytes: &'a [u8]) -> Result<Self, RuleSetError> {
        let field = try_from_bytes::<Str32>(0, Str32::SIZE, bytes)?;
        Ok(Self { field })
    }

    /// Serialize a constraint into a byte array.
    pub fn serialize(field: String) -> Result<Vec<u8>, RuleSetError> {
        let mut data = Vec::with_capacity(HEADER_SECTION + Str32::SIZE);

        // Header
        Header::serialize(ConstraintType::IsWallet, Str32::SIZE as u32, &mut data);

        // Constraint
        // - field
        let mut field_bytes = [0u8; Str32::SIZE];
        field_bytes[..field.len()].copy_from_slice(field.as_bytes());
        data.extend(field_bytes);

        Ok(data)
    }
}

impl<'a> Constraint<'a> for IsWallet<'a> {
    fn constraint_type(&self) -> ConstraintType {
        ConstraintType::IsWallet
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
        msg!("Validating IsWallet");

        let sysvar_instructions_info = match accounts.get(&sysvar::instructions::ID) {
            Some(sysvar_instructions_info) => *sysvar_instructions_info,
            _ => return RuleResult::Error(RuleSetError::MissingAccount.into()),
        };

        // Fetch the Pubkey of the caller from the payload.
        let caller = match payload.get_pubkey(&"caller".to_string()) {
            Some(pubkey) => pubkey,
            _ => return RuleResult::Error(RuleSetError::MissingPayloadValue.into()),
        };

        // If the program id of the current instruction is anything other than our program id
        // we know this is a CPI call from another program.
        let current_ix = get_instruction_relative(0, sysvar_instructions_info).unwrap();

        let is_cpi = !cmp_pubkeys(&current_ix.program_id, caller);

        // Get the `Pubkey` we are checking from the payload.
        let key = match payload.get_pubkey(&self.field.to_string()) {
            Some(pubkey) => pubkey,
            _ => return RuleResult::Error(RuleSetError::MissingPayloadValue.into()),
        };

        // Get the `AccountInfo` struct for the `Pubkey`.
        let account = match accounts.get(key) {
            Some(account) => *account,
            _ => return RuleResult::Error(RuleSetError::MissingAccount.into()),
        };

        // These checks can be replaced with a sys call to curve25519 once that feature activates.
        let system_program_owned = cmp_pubkeys(account.owner, &system_program::ID);

        // TODO: Uncomment call to `is_on_curve()` after on-curve sycall available.
        //(is_on_curve(key), self.to_error())

        if !is_cpi && system_program_owned {
            RuleResult::Success(self.constraint_type().to_error())
        } else {
            RuleResult::Failure(self.constraint_type().to_error())
        }
    }
}
