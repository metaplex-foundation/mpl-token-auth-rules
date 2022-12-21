use solana_program::pubkey::Pubkey;

pub const PREFIX: &str = "rule_set";
pub const STATE_PDA: &str = "rule_set_state";

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
