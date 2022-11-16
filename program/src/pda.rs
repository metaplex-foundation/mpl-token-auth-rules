use solana_program::pubkey::Pubkey;

pub const PREFIX: &str = "ruleset";

pub fn find_ruleset_address(creator: Pubkey, name: String) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[PREFIX.as_bytes(), creator.as_ref(), name.as_bytes()],
        &crate::id(),
    )
}
