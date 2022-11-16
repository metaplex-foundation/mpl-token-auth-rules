#![cfg(feature = "test-bpf")]

pub mod utils;

use solana_program_test::tokio;
use solana_sdk::{signature::Signer, transaction::Transaction};

use utils::program_test;

#[tokio::test]
async fn test_validator_transaction() {
    let mut context = program_test().start_with_context().await;

    let (ruleset_addr, _ruleset_bump) =
        authenticatooor::pda::find_ruleset_address(context.payer.pubkey(), "da rulez".to_string());

    let create_ix = authenticatooor::instruction::create(
        authenticatooor::id(),
        context.payer.pubkey(),
        ruleset_addr,
        "da rulez".to_string(),
    );

    let create_tx = Transaction::new_signed_with_payer(
        &[create_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    context
        .banks_client
        .process_transaction(create_tx)
        .await
        .expect("creation should succeed");
}
