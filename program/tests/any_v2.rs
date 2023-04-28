#![cfg(feature = "test-bpf")]

pub mod utils;

use mpl_token_auth_rules::{
    error::RuleSetError,
    instruction::{builders::ValidateBuilder, InstructionBuilder, ValidateArgs},
    payload::{Payload, PayloadType},
    state::{AdditionalSigner, Amount, Any, Operator, RuleSetV2},
};
use solana_program_test::tokio;
use solana_sdk::{signature::Signer, signer::keypair::Keypair};
use utils::{program_test, Operation, PayloadKey};

#[tokio::test]
async fn test_any_v2() {
    let mut context = program_test().start_with_context().await;

    // --------------------------------
    // Create RuleSet
    // --------------------------------
    // Create some rules.
    let adtl_signer = AdditionalSigner::serialize(Keypair::new().pubkey()).unwrap();
    let amount_check = Amount::serialize(PayloadKey::Amount.to_string(), Operator::Lt, 5).unwrap();

    let overall_rule = Any::serialize(&[&adtl_signer, &amount_check]).unwrap();

    // Create a RuleSet.
    let rule_set = RuleSetV2::serialize(
        context.payer.pubkey(),
        "test rule_set",
        &[Operation::Transfer {
            scenario: utils::TransferScenario::Holder,
        }
        .to_string()],
        &[&overall_rule],
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

    // Store a payload of data with the WRONG amount.
    let payload = Payload::from([(PayloadKey::Amount.to_string(), PayloadType::Number(5))]);

    // Create a `validate` instruction without the additional signer and sending WRONG amount.
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

    // Check that error is what we expect.  In this case we expect the last failure to roll up.
    assert_custom_error!(err, RuleSetError::AmountCheckFailed);

    // --------------------------------
    // Validate pass
    // --------------------------------
    // Store a payload of data with the correct amount.
    let payload = Payload::from([(PayloadKey::Amount.to_string(), PayloadType::Number(4))]);

    // Create a `validate` instruction without the additional signer but sending correct amount.
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

    // Validate Transfer operation since at least one Rule condition was true.
    process_passing_validate_ix!(&mut context, validate_ix, vec![], None).await;
}
