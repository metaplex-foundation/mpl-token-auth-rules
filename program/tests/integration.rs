#![cfg(feature = "test-bpf")]

pub mod utils;

use rmp_serde::Serializer;
use serde::Serialize;
use solana_program_test::tokio;
use solana_sdk::{signature::Signer, transaction::Transaction};
use token_authorization_rules::{
    state::{Operation, Rule, RuleSet},
    Payload, PayloadVec,
};
use utils::program_test;

#[tokio::test]
async fn test_validator_transaction() {
    let mut context = program_test().start_with_context().await;

    // Find RuleSet PDA.
    let (ruleset_addr, _ruleset_bump) = token_authorization_rules::pda::find_ruleset_address(
        context.payer.pubkey(),
        "da rulez".to_string(),
    );

    // Create some rules.
    let adtl_signer = Rule::AdditionalSigner {
        account: context.payer.pubkey(),
    };
    let adtl_signer2 = Rule::AdditionalSigner {
        account: context.payer.pubkey(),
    };
    let amount_check = Rule::Amount { amount: 2 };

    // Store the payloads that represent rule-specific data.
    let mut payloads = PayloadVec::new();
    payloads
        .add(&amount_check, Payload::Amount { amount: 2 })
        .unwrap();

    let first_rule = Rule::All {
        rules: vec![adtl_signer, adtl_signer2],
    };

    let overall_rule = Rule::All {
        rules: vec![first_rule, amount_check],
    };

    // Create a RuleSet.
    let mut rule_set = RuleSet::new();
    rule_set.add(Operation::Transfer, overall_rule);

    println!("{:#?}", rule_set);

    // Serialize the RuleSet using RMP serde.
    let mut serialized_data = Vec::new();
    rule_set
        .serialize(&mut Serializer::new(&mut serialized_data))
        .unwrap();

    // Create a `create` instruction.
    let create_ix = token_authorization_rules::instruction::create(
        token_authorization_rules::id(),
        context.payer.pubkey(),
        ruleset_addr,
        "da rulez".to_string(),
        serialized_data,
    );

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

    // Create a `validate` instruction.
    let validate_ix = token_authorization_rules::instruction::validate(
        token_authorization_rules::id(),
        context.payer.pubkey(),
        ruleset_addr,
        "da rulez".to_string(),
        Operation::Transfer,
        payloads,
        vec![],
        vec![],
    );

    // Add it to a transaction.
    let validate_tx = Transaction::new_signed_with_payer(
        &[validate_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    // Process the transaction.
    context
        .banks_client
        .process_transaction(validate_tx)
        .await
        .expect("validation should succeed");
}
