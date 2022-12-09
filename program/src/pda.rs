use solana_program::pubkey::Pubkey;

pub const PREFIX: &str = "rule_set";
pub const FREQ_PDA: &str = "frequency";

pub fn find_rule_set_address(creator: Pubkey, rule_set_name: String) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            PREFIX.as_bytes(),
            creator.as_ref(),
            rule_set_name.as_bytes(),
        ],
        &crate::id(),
    )
}

pub fn find_frequency_pda_address(
    creator: Pubkey,
    rule_set_name: String,
    freq_rule_name: String,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            FREQ_PDA.as_bytes(),
            creator.as_ref(),
            rule_set_name.as_bytes(),
            freq_rule_name.as_bytes(),
        ],
        &crate::id(),
    )
}
