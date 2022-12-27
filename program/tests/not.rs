#![cfg(feature = "test-bpf")]

pub mod utils;

use mpl_token_auth_rules::{
    error::RuleSetError,
    instruction::{builders::ValidateBuilder, InstructionBuilder, ValidateArgs},
    payload::{Payload, PayloadKey, PayloadType},
    state::{CompareOp, Rule, RuleSet},
};
use solana_program_test::tokio;
use solana_sdk::{signature::Signer, signer::keypair::Keypair};
use utils::{
    assert_rule_set_error, create_rule_set_on_chain, process_failing_validate_ix,
    process_passing_validate_ix, program_test, Operation,
};

#[tokio::test]
async fn test_not() {
    let mut context = program_test().start_with_context().await;
    // --------------------------------
    // Create RuleSet
    // --------------------------------
    // Create some rules.
    let amount_check = Rule::Amount {
        amount: 100,
        operator: CompareOp::Lt,
    };

    let not_amount_check = Rule::Not {
        rule: Box::new(amount_check),
    };

    // Create a RuleSet.
    let mut rule_set = RuleSet::new("test rule_set".to_string(), context.payer.pubkey());
    rule_set
        .add(Operation::Transfer.to_string(), not_amount_check)
        .unwrap();

    println!("{:#?}", rule_set);

    // Put the RuleSet on chain.
    let rule_set_addr =
        create_rule_set_on_chain(&mut context, rule_set, "test rule_set".to_string()).await;

    // --------------------------------
    // Validate fail
    // --------------------------------
    // Create a Keypair to simulate a token mint address.
    let mint = Keypair::new().pubkey();

    // Store a payload of data with an amount allowed by the Amount Rule (Amount Rule NOT'd).
    let payload = Payload::from([(PayloadKey::Amount, PayloadType::Number(2))]);

    // Create a `validate` instruction.
    let validate_ix = ValidateBuilder::new()
        .rule_set_pda(rule_set_addr)
        .mint(mint)
        .additional_rule_accounts(vec![])
        .build(ValidateArgs::V1 {
            operation: Operation::Transfer.to_string(),
            payload,
            update_rule_state: false,
        })
        .unwrap()
        .instruction();

    // Fail to validate Transfer operation because the Amount Rule was NOT'd.
    let err = process_failing_validate_ix(&mut context, validate_ix, vec![]).await;

    // Check that error is what we expect.
    assert_rule_set_error(err, RuleSetError::AmountCheckFailed);

    // --------------------------------
    // Validate pass
    // --------------------------------
    // Store a payload of data with an amount not allowed by the Amount Rule (Amount Rule NOT'd).
    let payload = Payload::from([(PayloadKey::Amount, PayloadType::Number(102))]);

    // Create a `validate` instruction without the additional signer but sending correct amount.
    let validate_ix = ValidateBuilder::new()
        .rule_set_pda(rule_set_addr)
        .mint(mint)
        .additional_rule_accounts(vec![])
        .build(ValidateArgs::V1 {
            operation: Operation::Transfer.to_string(),
            payload,
            update_rule_state: false,
        })
        .unwrap()
        .instruction();

    // Validate Transfer operation since because the Amount Rule was NOT'd.
    process_passing_validate_ix(&mut context, validate_ix, vec![]).await;
}
