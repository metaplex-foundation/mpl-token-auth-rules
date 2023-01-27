#![cfg(feature = "test-bpf")]

pub mod utils;

use mpl_token_auth_rules::{
    error::RuleSetError,
    instruction::{
        builders::{CreateOrUpdateBuilder, ValidateBuilder},
        CreateOrUpdateArgs, InstructionBuilder, ValidateArgs,
    },
    payload::{Payload, PayloadType},
    state::{CompareOp, Rule, RuleSetV1},
};
use rmp_serde::Serializer;
use serde::Serialize;
use solana_program::instruction::AccountMeta;
use solana_program_test::tokio;
use solana_sdk::{signature::Signer, signer::keypair::Keypair, transaction::Transaction};
use utils::{program_test, Operation, PayloadKey};

#[tokio::test]
#[should_panic]
async fn test_payer_not_signer_panics() {
    let mut context = program_test().start_with_context().await;

    // --------------------------------
    // Create RuleSet
    // --------------------------------
    // Create a Pass Rule.
    let pass_rule = Rule::Pass;

    // Create a RuleSet.
    let other_payer = Keypair::new();
    let mut rule_set = RuleSetV1::new("test rule_set".to_string(), other_payer.pubkey());
    rule_set
        .add(Operation::OwnerTransfer.to_string(), pass_rule)
        .unwrap();

    // Find RuleSet PDA.
    let (rule_set_addr, _rule_set_bump) = mpl_token_auth_rules::pda::find_rule_set_address(
        other_payer.pubkey(),
        "test rule_set".to_string(),
    );

    // Serialize the RuleSet using RMP serde.
    let mut serialized_rule_set = Vec::new();
    rule_set
        .serialize(&mut Serializer::new(&mut serialized_rule_set))
        .unwrap();

    // Create a `create` instruction with a payer that won't be a signer.
    let create_ix = CreateOrUpdateBuilder::new()
        .payer(other_payer.pubkey())
        .rule_set_pda(rule_set_addr)
        .build(CreateOrUpdateArgs::V1 {
            serialized_rule_set,
        })
        .unwrap()
        .instruction();

    // Add it to a transaction but don't add other payer as a signer.
    let create_tx = Transaction::new_signed_with_payer(
        &[create_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    // Process the transaction.  It will panic because of not enough signers.
    let _result = context.banks_client.process_transaction(create_tx).await;
}

#[tokio::test]
async fn test_composed_rule() {
    let mut context = program_test().start_with_context().await;

    // --------------------------------
    // Create RuleSet
    // --------------------------------
    // Create some rules.
    let adtl_signer = Rule::AdditionalSigner {
        account: context.payer.pubkey(),
    };

    // Second signer.
    let second_signer = Keypair::new();

    let adtl_signer2 = Rule::AdditionalSigner {
        account: second_signer.pubkey(),
    };
    let amount_check = Rule::Amount {
        amount: 1,
        operator: CompareOp::Eq,
        field: PayloadKey::Amount.to_string(),
    };
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
    let mut rule_set = RuleSetV1::new("test rule_set".to_string(), context.payer.pubkey());
    rule_set
        .add(Operation::OwnerTransfer.to_string(), overall_rule)
        .unwrap();

    println!("{:#?}", rule_set);

    // Put the RuleSet on chain.
    let rule_set_addr =
        create_rule_set_on_chain!(&mut context, rule_set, "test rule_set".to_string()).await;

    // --------------------------------
    // Validate fail missing account
    // --------------------------------
    // Create a Keypair to simulate a token mint address.
    let mint = Keypair::new().pubkey();

    // Store a payload of data with an amount not allowed by the Amount Rule (Amount Rule NOT'd).
    let payload = Payload::from([(PayloadKey::Amount.to_string(), PayloadType::Number(2))]);

    // Create a `validate` instruction WITHOUT the second signer.
    let validate_ix = ValidateBuilder::new()
        .rule_set_pda(rule_set_addr)
        .mint(mint)
        .additional_rule_accounts(vec![AccountMeta::new_readonly(
            context.payer.pubkey(),
            true,
        )])
        .build(ValidateArgs::V1 {
            operation: Operation::OwnerTransfer.to_string(),
            payload: payload.clone(),
            update_rule_state: false,
            rule_set_revision: None,
        })
        .unwrap()
        .instruction();

    // Fail to validate Transfer operation.
    let err = process_failing_validate_ix!(&mut context, validate_ix, vec![], None).await;

    // Check that error is what we expect.
    assert_custom_error!(err, RuleSetError::MissingAccount);

    // --------------------------------
    // Validate pass
    // --------------------------------
    // Create a `validate` instruction WITH the second signer.
    let validate_ix = ValidateBuilder::new()
        .rule_set_pda(rule_set_addr)
        .mint(mint)
        .additional_rule_accounts(vec![
            AccountMeta::new_readonly(context.payer.pubkey(), true),
            AccountMeta::new_readonly(second_signer.pubkey(), true),
        ])
        .build(ValidateArgs::V1 {
            operation: Operation::OwnerTransfer.to_string(),
            payload,
            update_rule_state: false,
            rule_set_revision: None,
        })
        .unwrap()
        .instruction();

    // Validate Transfer operation.
    process_passing_validate_ix!(&mut context, validate_ix, vec![&second_signer], None).await;

    // --------------------------------
    // Validate fail wrong amount
    // --------------------------------
    // Store a payload of data with an amount allowed by the Amount Rule (Amount Rule NOT'd).
    let payload = Payload::from([(PayloadKey::Amount.to_string(), PayloadType::Number(1))]);

    // Create a `validate` instruction WITH the second signer.  Will fail as Amount Rule is NOT'd.
    let validate_ix = ValidateBuilder::new()
        .rule_set_pda(rule_set_addr)
        .mint(mint)
        .additional_rule_accounts(vec![
            AccountMeta::new_readonly(context.payer.pubkey(), true),
            AccountMeta::new_readonly(second_signer.pubkey(), true),
        ])
        .build(ValidateArgs::V1 {
            operation: Operation::OwnerTransfer.to_string(),
            payload,
            update_rule_state: false,
            rule_set_revision: None,
        })
        .unwrap()
        .instruction();

    // Fail to validate Transfer operation.
    let err =
        process_failing_validate_ix!(&mut context, validate_ix, vec![&second_signer], None).await;

    // Check that error is what we expect.
    assert_custom_error!(err, RuleSetError::AmountCheckFailed);
}

#[tokio::test]
async fn test_rule_set_creation_wallet_fails() {
    let mut context = program_test().start_with_context().await;

    // --------------------------------
    // Create RuleSet
    // --------------------------------
    // Create some rules.
    let adtl_signer = Rule::AdditionalSigner {
        account: context.payer.pubkey(),
    };

    // Create a RuleSet.
    let mut rule_set = RuleSetV1::new("test rule_set".to_string(), context.payer.pubkey());
    rule_set
        .add(Operation::OwnerTransfer.to_string(), adtl_signer)
        .unwrap();

    // Serialize the RuleSet using RMP serde.
    let mut serialized_rule_set = Vec::new();
    rule_set
        .serialize(&mut Serializer::new(&mut serialized_rule_set))
        .unwrap();

    // --------------------------------
    // Fail on-chain creation
    // --------------------------------
    // Create a `create` instruction.
    let create_ix = CreateOrUpdateBuilder::new()
        .payer(context.payer.pubkey())
        .rule_set_pda(Keypair::new().pubkey())
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
    let err = context
        .banks_client
        .process_transaction(create_tx)
        .await
        .expect_err("Creation should fail");

    // Check that error is what we expect.
    assert_custom_error!(err, RuleSetError::DerivedKeyInvalid);
}

#[tokio::test]
async fn test_rule_set_creation_wrong_pda_fails() {
    let mut context = program_test().start_with_context().await;

    // --------------------------------
    // Create RuleSet
    // --------------------------------
    // Create some rules.
    let adtl_signer = Rule::AdditionalSigner {
        account: context.payer.pubkey(),
    };

    // Create a RuleSet.
    let mut rule_set = RuleSetV1::new("test rule_set".to_string(), context.payer.pubkey());
    rule_set
        .add(Operation::OwnerTransfer.to_string(), adtl_signer)
        .unwrap();

    // Serialize the RuleSet using RMP serde.
    let mut serialized_rule_set = Vec::new();
    rule_set
        .serialize(&mut Serializer::new(&mut serialized_rule_set))
        .unwrap();

    // --------------------------------
    // Fail on-chain creation
    // --------------------------------
    // Find RuleSet PDA using WRONG name for seed.
    let (wrong_rule_set_addr, _rule_set_bump) = mpl_token_auth_rules::pda::find_rule_set_address(
        context.payer.pubkey(),
        "WRONG NAME".to_string(),
    );

    // Create a `create` instruction.
    let create_ix = CreateOrUpdateBuilder::new()
        .payer(context.payer.pubkey())
        .rule_set_pda(wrong_rule_set_addr)
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
    let err = context
        .banks_client
        .process_transaction(create_tx)
        .await
        .expect_err("Creation should fail");

    // Check that error is what we expect.
    assert_custom_error!(err, RuleSetError::DerivedKeyInvalid);
}

#[tokio::test]
async fn test_rule_set_validate_wallet_fails() {
    let mut context = program_test().start_with_context().await;

    // --------------------------------
    // Validate fail incorrect owner
    // --------------------------------
    // Create a Keypair to simulate a token mint address.
    let mint = Keypair::new().pubkey();

    // Create a `validate` instruction WITH the second signer.
    let validate_ix = ValidateBuilder::new()
        .rule_set_pda(Keypair::new().pubkey())
        .mint(mint)
        .additional_rule_accounts(vec![AccountMeta::new_readonly(
            context.payer.pubkey(),
            true,
        )])
        .build(ValidateArgs::V1 {
            operation: Operation::OwnerTransfer.to_string(),
            payload: Payload::default(),
            update_rule_state: false,
            rule_set_revision: None,
        })
        .unwrap()
        .instruction();

    // Fail to validate operation.
    let err = process_failing_validate_ix!(&mut context, validate_ix, vec![], None).await;

    // Check that error is what we expect.
    assert_custom_error!(err, RuleSetError::IncorrectOwner);
}

#[tokio::test]
async fn test_rule_set_validate_uninitialized_pda_fails() {
    let mut context = program_test().start_with_context().await;

    // Find RuleSet PDA.
    let (rule_set_addr, _rule_set_bump) = mpl_token_auth_rules::pda::find_rule_set_address(
        context.payer.pubkey(),
        "test rule_set".to_string(),
    );

    // --------------------------------
    // Validate fail incorrect owner
    // --------------------------------
    // Create a Keypair to simulate a token mint address.
    let mint = Keypair::new().pubkey();

    // Create a `validate` instruction WITH the second signer.
    let validate_ix = ValidateBuilder::new()
        .rule_set_pda(rule_set_addr)
        .mint(mint)
        .additional_rule_accounts(vec![AccountMeta::new_readonly(
            context.payer.pubkey(),
            true,
        )])
        .build(ValidateArgs::V1 {
            operation: Operation::OwnerTransfer.to_string(),
            payload: Payload::default(),
            update_rule_state: false,
            rule_set_revision: None,
        })
        .unwrap()
        .instruction();

    // Fail to validate operation.
    let err = process_failing_validate_ix!(&mut context, validate_ix, vec![], None).await;

    // Check that error is what we expect.
    assert_custom_error!(err, RuleSetError::IncorrectOwner);
}
