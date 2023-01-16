//! The helper functions for the PDA accounts.
use solana_program::pubkey::Pubkey;

/// The string prefix for Rule Set PDA seeds.
pub const PREFIX: &str = "rule_set";

/// The string prefix for Rule Set State PDA seeds.
pub const STATE_PDA: &str = "rule_set_state";

/// Find the PDA for a Rule Set account.
pub fn find_rule_set_address(creator: Pubkey, rule_set_name: String) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            PREFIX.as_bytes(),
            creator.as_ref(),
            rule_set_name.as_bytes(),
        ],
        &crate::ID,
    )
}

/// Find the PDA for a Rule Set State account.
pub fn find_rule_set_state_address(
    creator: Pubkey,
    rule_set_name: String,
    mint: Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            STATE_PDA.as_bytes(),
            creator.as_ref(),
            rule_set_name.as_bytes(),
            mint.as_ref(),
        ],
        &crate::ID,
    )
}

/// Find the PDA for the Rule Set buffer account.
pub fn find_buffer_address(creator: Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[PREFIX.as_bytes(), creator.as_ref()], &crate::ID)
}
