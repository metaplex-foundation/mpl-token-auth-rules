#![cfg(feature = "test-sbf")]

pub mod utils;

use mpl_token_auth_rules::{
    error::RuleSetError,
    instruction::{builders::ValidateBuilder, InstructionBuilder, ValidateArgs},
    payload::{Payload, PayloadType},
    state::{Amount, Operator, RuleSetV2},
};
use solana_program_test::tokio;
use solana_sdk::{signature::Signer, signer::keypair::Keypair};
use utils::{program_test, Operation, PayloadKey};

#[tokio::test]
async fn test_less_than_amount_v2() {
    parametric_amount_check(Operator::Lt, 100, 100, 99).await;
}

#[tokio::test]
async fn test_less_than_or_equal_to_amount_v2() {
    parametric_amount_check(Operator::LtEq, 100, 101, 100).await;
}

#[tokio::test]
async fn equal_to_amount_fail_less_than_v2() {
    parametric_amount_check(Operator::Eq, 100, 99, 100).await;
}

#[tokio::test]
async fn equal_to_amount_fail_greater_than_v2() {
    parametric_amount_check(Operator::Eq, 100, 101, 100).await;
}

#[tokio::test]
async fn test_greater_than_or_equal_to_amount_v2() {
    parametric_amount_check(Operator::GtEq, 100, 99, 100).await;
}

#[tokio::test]
async fn test_greater_than_amount_v2() {
    parametric_amount_check(Operator::Gt, 100, 100, 101).await;
}

async fn parametric_amount_check(
    operator: Operator,
    amount: u64,
    fail_amount: u64,
    pass_amount: u64,
) {
    let mut context = program_test().start_with_context().await;
    // --------------------------------
    // Create RuleSet
    // --------------------------------
    // Create a rule.
    let less_than_amount_check =
        Amount::serialize(PayloadKey::Amount.to_string(), operator, amount).unwrap();

    // Create a RuleSet.
    let rule_set = RuleSetV2::serialize(
        context.payer.pubkey(),
        "test rule_set",
        &[Operation::Transfer {
            scenario: utils::TransferScenario::Holder,
        }
        .to_string()],
        &[&less_than_amount_check],
    )
    .unwrap();

    // Put the RuleSet on chain.
    let rule_set_addr =
        create_rule_set_on_chain_serialized!(&mut context, rule_set, "test rule_set".to_string())
            .await;

    // --------------------------------
    // Validate fail
    // --------------------------------
    // Create a Keypair to simulate a token mint address.
    let mint = Keypair::new().pubkey();

    // Store a payload of data with an amount not allowed by the Amount Rule.
    let payload = Payload::from([(
        PayloadKey::Amount.to_string(),
        PayloadType::Number(fail_amount),
    )]);

    // Create a `validate` instruction.
    let validate_ix = ValidateBuilder::new()
        .rule_set_pda(rule_set_addr)
        .mint(mint)
        .additional_rule_accounts(vec![])
        .build(ValidateArgs::V1 {
            operation: Operation::Transfer {
                scenario: utils::TransferScenario::Holder,
            }
            .to_string(),
            payload,
            update_rule_state: false,
            rule_set_revision: None,
        })
        .unwrap()
        .instruction();

    // Fail to validate Transfer operation.
    let err = process_failing_validate_ix!(&mut context, validate_ix, vec![], None).await;

    // Check that error is what we expect.
    assert_custom_error!(err, RuleSetError::AmountCheckFailed);

    // --------------------------------
    // Validate pass
    // --------------------------------
    // Store a payload of data with an amount allowed by the Amount Rule.
    let payload = Payload::from([(
        PayloadKey::Amount.to_string(),
        PayloadType::Number(pass_amount),
    )]);

    // Create a `validate` instruction.
    let validate_ix = ValidateBuilder::new()
        .rule_set_pda(rule_set_addr)
        .mint(mint)
        .additional_rule_accounts(vec![])
        .build(ValidateArgs::V1 {
            operation: Operation::Transfer {
                scenario: utils::TransferScenario::Holder,
            }
            .to_string(),
            payload,
            update_rule_state: false,
            rule_set_revision: None,
        })
        .unwrap()
        .instruction();

    // Validate Transfer operation since because the Amount Rule was NOT'd.
    process_passing_validate_ix!(&mut context, validate_ix, vec![], None).await;
}
