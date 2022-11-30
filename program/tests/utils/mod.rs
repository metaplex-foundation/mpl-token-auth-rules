use solana_program_test::ProgramTest;

pub fn program_test() -> ProgramTest {
    ProgramTest::new(
        "token_authorization_rules",
        token_authorization_rules::id(),
        None,
    )
}
