#![cfg(feature = "test-bpf")]

pub mod utils;

use mpl_token_auth_rules::{
    error::RuleSetError,
    instruction::{
        builders::{CreateOrUpdateBuilder, ValidateBuilder},
        CreateOrUpdateArgs, InstructionBuilder, ValidateArgs,
    },
    payload::{Payload, PayloadKey, PayloadType},
    state::{CompareOp, Rule, RuleSet},
};
use solana_program::instruction::AccountMeta;
use solana_program_test::{tokio, BanksClientError};
use solana_sdk::{
    signature::Signer,
    signer::keypair::Keypair,
    transaction::{Transaction, TransactionError},
};
use utils::{
    assert_rule_set_error, create_rule_set_on_chain, process_failing_validate_ix,
    process_passing_validate_ix, program_test, Operation,
};

#[tokio::test]
async fn test_payer_not_signer_fails() {
    let mut context = program_test().start_with_context().await;

    // --------------------------------
    // Create fail unsigned tx
    // --------------------------------
    // Find RuleSet PDA.
    let (rule_set_addr, _rule_set_bump) = mpl_token_auth_rules::pda::find_rule_set_address(
        context.payer.pubkey(),
        "test rule_set".to_string(),
    );

    // Create a `create` instruction.
    let create_ix = CreateOrUpdateBuilder::new()
        .payer(context.payer.pubkey())
        .rule_set_pda(rule_set_addr)
        .build(CreateOrUpdateArgs::V1 {
            serialized_rule_set: vec![],
        })
        .unwrap()
        .instruction();

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

    // --------------------------------
    // Validate fail unsigned tx
    // --------------------------------
    // Create a Keypair to simulate a token mint address.
    let mint = Keypair::new().pubkey();

    // Create a `validate` instruction.
    let validate_ix = ValidateBuilder::new()
        .rule_set_pda(rule_set_addr)
        .mint(mint)
        .additional_rule_accounts(vec![])
        .build(ValidateArgs::V1 {
            operation: Operation::Transfer.to_string(),
            payload: Payload::default(),
            update_rule_state: false,
        })
        .unwrap()
        .instruction();

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
async fn test_additional_signer_and_not_amount() {
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
    let mut rule_set = RuleSet::new("test rule_set".to_string(), context.payer.pubkey());
    rule_set
        .add(Operation::Transfer.to_string(), overall_rule)
        .unwrap();

    println!("{:#?}", rule_set);

    // Put the RuleSet on chain.
    let rule_set_addr =
        create_rule_set_on_chain(&mut context, rule_set, "test rule_set".to_string()).await;

    // --------------------------------
    // Validate fail missing account
    // --------------------------------
    // Create a Keypair to simulate a token mint address.
    let mint = Keypair::new().pubkey();

    // Store a payload of data with an amount not allowed by the Amount Rule (Amount Rule NOT'd).
    let payload = Payload::from([(PayloadKey::Amount, PayloadType::Number(2))]);

    // Create a `validate` instruction WITHOUT the second signer.
    let validate_ix = ValidateBuilder::new()
        .rule_set_pda(rule_set_addr)
        .mint(mint)
        .additional_rule_accounts(vec![AccountMeta::new_readonly(
            context.payer.pubkey(),
            true,
        )])
        .build(ValidateArgs::V1 {
            operation: Operation::Transfer.to_string(),
            payload: payload.clone(),
            update_rule_state: false,
        })
        .unwrap()
        .instruction();

    // Fail to validate Transfer operation.
    let err = process_failing_validate_ix(&mut context, validate_ix, vec![]).await;

    // Check that error is what we expect.
    assert_rule_set_error(err, RuleSetError::MissingAccount);

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
            operation: Operation::Transfer.to_string(),
            payload,
            update_rule_state: false,
        })
        .unwrap()
        .instruction();

    // Validate Transfer operation.
    process_passing_validate_ix(&mut context, validate_ix, vec![&second_signer]).await;

    // --------------------------------
    // Validate fail wrong amount
    // --------------------------------
    // Store a payload of data with an amount allowed by the Amount Rule (Amount Rule NOT'd).
    let payload = Payload::from([(PayloadKey::Amount, PayloadType::Number(1))]);

    // Create a `validate` instruction WITH the second signer.  Will fail as Amount Rule is NOT'd.
    let validate_ix = ValidateBuilder::new()
        .rule_set_pda(rule_set_addr)
        .mint(mint)
        .additional_rule_accounts(vec![
            AccountMeta::new_readonly(context.payer.pubkey(), true),
            AccountMeta::new_readonly(second_signer.pubkey(), true),
        ])
        .build(ValidateArgs::V1 {
            operation: Operation::Transfer.to_string(),
            payload,
            update_rule_state: false,
        })
        .unwrap()
        .instruction();

    // Fail to validate Transfer operation.
    let err = process_failing_validate_ix(&mut context, validate_ix, vec![&second_signer]).await;

    // Check that error is what we expect.
    assert_rule_set_error(err, RuleSetError::AmountCheckFailed);
}

#[tokio::test]
async fn test_update_ruleset() {
    let mut context = program_test().start_with_context().await;

    // --------------------------------
    // Create RuleSet
    // --------------------------------
    // Create a Pass Rule.
    let pass_rule = Rule::Pass;

    // Create a RuleSet.
    let mut rule_set = RuleSet::new("test rule_set".to_string(), context.payer.pubkey());
    rule_set
        .add(Operation::Transfer.to_string(), pass_rule)
        .unwrap();

    // Put the RuleSet on chain.
    let _rule_set_addr =
        create_rule_set_on_chain(&mut context, rule_set, "test rule_set".to_string()).await;

    // --------------------------------
    // Update RuleSet
    // --------------------------------
    // Create some other rules.
    let adtl_signer = Rule::AdditionalSigner {
        account: context.payer.pubkey(),
    };

    let amount_check = Rule::Amount {
        amount: 1,
        operator: CompareOp::Eq,
    };

    let overall_rule = Rule::All {
        rules: vec![adtl_signer, amount_check],
    };

    // Create a new RuleSet.
    let mut rule_set = RuleSet::new("test rule_set".to_string(), context.payer.pubkey());
    rule_set
        .add(Operation::Transfer.to_string(), overall_rule)
        .unwrap();

    // Put the updated RuleSet on chain.
    let _rule_set_addr =
        create_rule_set_on_chain(&mut context, rule_set, "test rule_set".to_string()).await;
}
