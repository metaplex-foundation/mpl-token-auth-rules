use solana_program_test::ProgramTest;

pub fn program_test() -> ProgramTest {
    ProgramTest::new(
        "token-authorization-rules",
        token_authorization_rules::id(),
        None,
    )
}
