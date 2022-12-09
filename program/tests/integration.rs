#![cfg(feature = "test-bpf")]

pub mod utils;

use mpl_token_auth_rules::{
    error::RuleSetError,
    state::{Operation, Rule, RuleSet},
    Payload,
};
use num_traits::cast::FromPrimitive;
use rmp_serde::Serializer;
use serde::Serialize;
use solana_program::instruction::InstructionError;
use solana_program_test::{tokio, BanksClientError};
use solana_sdk::{
    signature::Signer,
    signer::keypair::Keypair,
    transaction::{Transaction, TransactionError},
};
use utils::program_test;

#[tokio::test]
async fn test_payer_not_signer_fails() {
    let mut context = program_test().start_with_context().await;

    // Find RuleSet PDA.
    let (rule_set_addr, _rule_set_bump) = mpl_token_auth_rules::pda::find_rule_set_address(
        context.payer.pubkey(),
        "test rule_set".to_string(),
    );

    // Create a `create` instruction.
    let create_ix = mpl_token_auth_rules::instruction::create(
        mpl_token_auth_rules::id(),
        context.payer.pubkey(),
        rule_set_addr,
        "test rule_set".to_string(),
        vec![],
    );

    // Add it to a non-signed transaction.
    let create_tx = Transaction::new_with_payer(&[create_ix], Some(&context.payer.pubkey()));

    // Process the transaction.
    let err = context
        .banks_client
        .process_transaction(create_tx)
        .await
        .expect_err("creation should fail");

    // Deconstruct the error code and make sure it is what we expect.
    match err {
        BanksClientError::TransactionError(TransactionError::SignatureFailure) => (),
        _ => panic!("Unexpected error {:?}", err),
    }

    // Create a `validate` instruction.
    let validate_ix = mpl_token_auth_rules::instruction::validate(
        mpl_token_auth_rules::id(),
        context.payer.pubkey(),
        rule_set_addr,
        "test rule_set".to_string(),
        Operation::Transfer,
        Payload::default(),
        vec![],
        vec![],
    );

    // Add it to a non-signed transaction.
    let validate_tx = Transaction::new_with_payer(&[validate_ix], Some(&context.payer.pubkey()));

    // Process the transaction.
    let err = context
        .banks_client
        .process_transaction(validate_tx)
        .await
        .expect_err("validation should fail");

    // Deconstruct the error code and make sure it is what we expect.
    match err {
        BanksClientError::TransactionError(TransactionError::SignatureFailure) => (),
        _ => panic!("Unexpected error {:?}", err),
    }
}

#[tokio::test]
async fn test_additional_signer_and_amount() {
    let mut context = program_test().start_with_context().await;

    // Find RuleSet PDA.
    let (rule_set_addr, _rule_set_bump) = mpl_token_auth_rules::pda::find_rule_set_address(
        context.payer.pubkey(),
        "test rule_set".to_string(),
    );

    // Second signer.
    let second_signer = Keypair::new();

    // Create some rules.
    let adtl_signer = Rule::AdditionalSigner {
        account: context.payer.pubkey(),
    };
    let adtl_signer2 = Rule::AdditionalSigner {
        account: second_signer.pubkey(),
    };
    let amount_check = Rule::Amount { amount: 1 };
    let not_amount_check = Rule::Not {
        rule: Box::new(amount_check),
    };

    let first_rule = Rule::All {
        rules: vec![adtl_signer, adtl_signer2],
    };

    let overall_rule = Rule::All {
        rules: vec![first_rule, not_amount_check],
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
    let create_ix = mpl_token_auth_rules::instruction::create(
        mpl_token_auth_rules::id(),
        context.payer.pubkey(),
        rule_set_addr,
        "test rule_set".to_string(),
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

    // Store the payload of data to validate against the rule definition.
    let payload = Payload::new(None, None, Some(2), None);

    // Create a `validate` instruction WITHOUT the second signer.
    let validate_ix = mpl_token_auth_rules::instruction::validate(
        mpl_token_auth_rules::id(),
        context.payer.pubkey(),
        rule_set_addr,
        "test rule_set".to_string(),
        Operation::Transfer,
        payload.clone(),
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
    let err = context
        .banks_client
        .process_transaction(validate_tx)
        .await
        .expect_err("validation should fail");

    // Deconstruct the error code and make sure it is what we expect.
    match err {
        BanksClientError::TransactionError(TransactionError::InstructionError(
            _,
            InstructionError::Custom(val),
        )) => {
            let rule_set_error = RuleSetError::from_u32(val).unwrap();
            assert_eq!(rule_set_error, RuleSetError::AdditionalSignerCheckFailed);
        }
        _ => panic!("Unexpected error {:?}", err),
    }

    // Create a `validate` instruction WITH the second signer.
    let validate_ix = mpl_token_auth_rules::instruction::validate(
        mpl_token_auth_rules::id(),
        context.payer.pubkey(),
        rule_set_addr,
        "test rule_set".to_string(),
        Operation::Transfer,
        payload,
        vec![second_signer.pubkey()],
        vec![],
    );

    // Add it to a transaction.
    let validate_tx = Transaction::new_signed_with_payer(
        &[validate_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &second_signer],
        context.last_blockhash,
    );

    // Process the transaction, this time it should succeed.
    context
        .banks_client
        .process_transaction(validate_tx)
        .await
        .expect("validation should succeed");

    // Store a payload of data with the WRONG amount.
    let payload = Payload::new(None, None, Some(1), None);

    // Create a `validate` instruction.
    let validate_ix = mpl_token_auth_rules::instruction::validate(
        mpl_token_auth_rules::id(),
        context.payer.pubkey(),
        rule_set_addr,
        "test rule_set".to_string(),
        Operation::Transfer,
        payload,
        vec![second_signer.pubkey()],
        vec![],
    );

    // Add it to a transaction.
    let validate_tx = Transaction::new_signed_with_payer(
        &[validate_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &second_signer],
        context.last_blockhash,
    );

    // Process the transaction.
    let err = context
        .banks_client
        .process_transaction(validate_tx)
        .await
        .expect_err("validation should fail");

    // Deconstruct the error code and make sure it is what we expect.
    match err {
        BanksClientError::TransactionError(TransactionError::InstructionError(
            _,
            InstructionError::Custom(val),
        )) => {
            let rule_set_error = RuleSetError::from_u32(val).unwrap();
            assert_eq!(rule_set_error, RuleSetError::AmountCheckFailed);
        }
        _ => panic!("Unexpected error {:?}", err),
    }
}
