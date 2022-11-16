use solana_program_test::ProgramTest;

pub fn program_test() -> ProgramTest {
    ProgramTest::new("authenticatooor", authenticatooor::id(), None)
}
