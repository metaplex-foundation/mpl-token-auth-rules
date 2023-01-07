use mpl_token_auth_rules::{
    error::RuleSetError,
    instruction::{builders::CreateOrUpdateBuilder, CreateOrUpdateArgs, InstructionBuilder},
    state::RuleSet,
};
use num_derive::ToPrimitive;
use num_traits::cast::FromPrimitive;
use rmp_serde::Serializer;
use serde::Serialize;
use solana_program::{
    instruction::{Instruction, InstructionError},
    program_error::ProgramError,
    pubkey::Pubkey,
};
use solana_program_test::{BanksClientError, ProgramTest, ProgramTestContext};
use solana_sdk::{
    signature::Signer,
    signer::keypair::Keypair,
    transaction::{Transaction, TransactionError},
};

#[repr(C)]
#[derive(ToPrimitive)]
pub enum Operation {
    OwnerTransfer,
    Delegate,
    SaleTransfer,
}

impl ToString for Operation {
    fn to_string(&self) -> String {
        match self {
            Operation::OwnerTransfer => "OwnerTransfer".to_string(),
            Operation::Delegate => "Delegate".to_string(),
            Operation::SaleTransfer => "SaleTransfer".to_string(),
        }
    }
}

#[repr(C)]
#[derive(ToPrimitive)]
pub enum PayloadKey {
    /// The amount being transferred.
    Amount,
    /// The authority of a transfer, e.g. the delegate of token.
    Authority,
    /// Seeds for a PDA authority of the operation, e.g. when the authority is a PDA.
    AuthoritySeeds,
    /// The source of the operation, e.g. the owner initiating a transfer.
    Source,
    /// Seeds for a PDA source of the operation, e.g. when the source is a PDA.
    SourceSeeds,
    /// The destination of the operation, e.g. the recipient of a transfer.
    Destination,
    /// Seeds for a PDA destination of the operation, e.g. when the recipient is a PDA.
    DestinationSeeds,
}

impl ToString for PayloadKey {
    fn to_string(&self) -> String {
        match self {
            PayloadKey::Amount => "Amount".to_string(),
            PayloadKey::Authority => "Authority".to_string(),
            PayloadKey::AuthoritySeeds => "AuthoritySeeds".to_string(),
            PayloadKey::Source => "Source".to_string(),
            PayloadKey::SourceSeeds => "SourceSeeds".to_string(),
            PayloadKey::Destination => "Destination".to_string(),
            PayloadKey::DestinationSeeds => "DestinationSeeds".to_string(),
        }
    }
}

pub fn program_test() -> ProgramTest {
    ProgramTest::new("mpl_token_auth_rules", mpl_token_auth_rules::id(), None)
}

pub async fn create_rule_set_on_chain(
    context: &mut ProgramTestContext,
    rule_set: RuleSet,
    rule_set_name: String,
) -> Pubkey {
    // Find RuleSet PDA.
    let (rule_set_addr, _rule_set_bump) =
        mpl_token_auth_rules::pda::find_rule_set_address(context.payer.pubkey(), rule_set_name);

    // Serialize the RuleSet using RMP serde.
    let mut serialized_rule_set = Vec::new();
    rule_set
        .serialize(&mut Serializer::new(&mut serialized_rule_set))
        .unwrap();

    // Create a `create` instruction.
    let create_ix = CreateOrUpdateBuilder::new()
        .payer(context.payer.pubkey())
        .rule_set_pda(rule_set_addr)
        .build(CreateOrUpdateArgs::V1 {
            serialized_rule_set,
        })
        .unwrap()
        .instruction();

    // Add it to a transaction.
    let create_tx = Transaction::new_signed_with_payer(
        &[create_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    // Process the transaction.
    context
        .banks_client
        .process_transaction(create_tx)
        .await
        .expect("creation should succeed");

    rule_set_addr
}

pub async fn process_passing_validate_ix(
    context: &mut ProgramTestContext,
    validate_ix: Instruction,
    additional_signers: Vec<&Keypair>,
) {
    let mut signing_keypairs = vec![&context.payer];
    signing_keypairs.extend(additional_signers);

    // Add ix to a transaction.
    let validate_tx = Transaction::new_signed_with_payer(
        &[validate_ix],
        Some(&context.payer.pubkey()),
        &signing_keypairs,
        context.last_blockhash,
    );

    // Process the transaction.
    context
        .banks_client
        .process_transaction(validate_tx)
        .await
        .expect("Validation should succeed");
}

pub async fn process_failing_validate_ix(
    context: &mut ProgramTestContext,
    validate_ix: Instruction,
    additional_signers: Vec<&Keypair>,
) -> BanksClientError {
    let mut signing_keypairs = vec![&context.payer];
    signing_keypairs.extend(additional_signers);

    // Add ix to a transaction.
    let validate_tx = Transaction::new_signed_with_payer(
        &[validate_ix],
        Some(&context.payer.pubkey()),
        &signing_keypairs,
        context.last_blockhash,
    );

    // Process the transaction.
    context
        .banks_client
        .process_transaction(validate_tx)
        .await
        .expect_err("validation should fail")
}

pub fn assert_rule_set_error(err: BanksClientError, rule_set_error: RuleSetError) {
    // Deconstruct the error code and make sure it is what we expect.
    match err {
        BanksClientError::TransactionError(TransactionError::InstructionError(
            _,
            InstructionError::Custom(val),
        )) => {
            let deconstructed_err = RuleSetError::from_u32(val).unwrap();
            assert_eq!(deconstructed_err, rule_set_error);
        }
        _ => panic!("Unexpected error {:?}", err),
    }
}

pub fn assert_program_error(err: BanksClientError, program_error: ProgramError) {
    // Deconstruct the error code and make sure it is what we expect.
    match err {
        BanksClientError::TransactionError(TransactionError::InstructionError(_, err)) => {
            assert_eq!(
                ProgramError::try_from(err)
                    .expect("Could not convert InstructionError to ProgramError"),
                program_error
            );
        }
        _ => panic!("Unexpected error {:?}", err),
    }
}
