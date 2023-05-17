#![cfg(feature = "test-bpf")]

pub mod utils;

use mpl_token_auth_rules::{
    error::RuleSetError,
    instruction::{builders::ValidateBuilder, InstructionBuilder, ValidateArgs},
    payload::{Payload, PayloadType},
    state::{CompareOp, Rule, RuleSetV1},
};
use solana_program_test::tokio;
use solana_sdk::{instruction::AccountMeta, signature::Signer, signer::keypair::Keypair};
use utils::{program_test, Operation, PayloadKey};

#[tokio::test]
async fn correct_rule_used_for_each_operation() {
    let mut context = program_test().start_with_context().await;

    // --------------------------------
    // Create RuleSet
    // --------------------------------
    // Create rules.
    let additional_signer = Keypair::new();
    let adtl_signer_rule = Rule::AdditionalSigner {
        account: additional_signer.pubkey(),
    };

    let amount_check = Rule::Amount {
        amount: 10,
        operator: CompareOp::Lt,
        field: PayloadKey::Amount.to_string(),
    };

    // Create a RuleSet.
    let mut rule_set = RuleSetV1::new("test rule_set".to_string(), context.payer.pubkey());

    // Use different rules for each operation.
    rule_set
        .add(
            Operation::Transfer {
                scenario: utils::TransferScenario::Holder,
            }
            .to_string(),
            adtl_signer_rule,
        )
        .unwrap();

    rule_set
        .add(
            Operation::Delegate {
                scenario: utils::DelegateScenario::Token(utils::TokenDelegateRole::Sale),
            }
            .to_string(),
            amount_check,
        )
        .unwrap();

    // Put the RuleSet on chain.
    let rule_set_addr =
        create_rule_set_on_chain!(&mut context, rule_set.clone(), "test rule_set".to_string())
            .await;

    // --------------------------------
    // Validate that when using first operation, we fail based on inputs that would pass the second
    // operation.
    // --------------------------------
    // Create a Keypair to simulate a token mint address.
    let mint = Keypair::new().pubkey();

    // Store a payload of data with an amount allowed by the the second operation's rule, but
    // ignored by the `SimpleOwnerTransfer` operation's rule.
    let payload = Payload::from([(PayloadKey::Amount.to_string(), PayloadType::Number(2))]);

    // Create a `validate` instruction with the additional signer pubkey added but not sent as a
    // signer.  Send the first operation.
    let validate_ix = ValidateBuilder::new()
        .rule_set_pda(rule_set_addr)
        .mint(mint)
        .additional_rule_accounts(vec![AccountMeta::new_readonly(
            additional_signer.pubkey(),
            false,
        )])
        .build(ValidateArgs::V1 {
            operation: Operation::Transfer {
                scenario: utils::TransferScenario::Holder,
            }
            .to_string(),
            payload: payload.clone(),
            update_rule_state: false,
            rule_set_revision: None,
        })
        .unwrap()
        .instruction();

    // Fail to validate Transfer operation.
    let err = process_failing_validate_ix!(&mut context, validate_ix, vec![], None).await;

    // Check that error is what we expect.
    assert_custom_error!(err, RuleSetError::AdditionalSignerCheckFailed);

    // --------------------------------
    // Validate that when using first operation, we pass based on inputs that would fail the second
    // operation.
    // --------------------------------
    // Store a payload of data with an amount not allowed by the the second operation's rule, but
    // ignored by the first operation's rule.
    let payload = Payload::from([(PayloadKey::Amount.to_string(), PayloadType::Number(200))]);

    // Create a `validate` instruction with the additional signer pubkey added as a signer.  Send
    // the first operation.
    let validate_ix = ValidateBuilder::new()
        .rule_set_pda(rule_set_addr)
        .mint(mint)
        .additional_rule_accounts(vec![AccountMeta::new_readonly(
            additional_signer.pubkey(),
            true,
        )])
        .build(ValidateArgs::V1 {
            operation: Operation::Transfer {
                scenario: utils::TransferScenario::Holder,
            }
            .to_string(),
            payload: payload.clone(),
            update_rule_state: false,
            rule_set_revision: None,
        })
        .unwrap()
        .instruction();

    // Validate Transfer operation.
    process_passing_validate_ix!(&mut context, validate_ix, vec![&additional_signer], None).await;

    // --------------------------------
    // Validate that when using second operation, we fail based on inputs that would pass the first
    // operation.
    // --------------------------------
    // Store a payload of data with an amount not allowed by the the second operation's rule, but ignored
    // by the first operation's rule.
    let payload = Payload::from([(PayloadKey::Amount.to_string(), PayloadType::Number(200))]);

    // Create a `validate` instruction with the additional signer pubkey added as a signer.  Send
    // the second operation.
    let validate_ix = ValidateBuilder::new()
        .rule_set_pda(rule_set_addr)
        .mint(mint)
        .additional_rule_accounts(vec![AccountMeta::new_readonly(
            additional_signer.pubkey(),
            true,
        )])
        .build(ValidateArgs::V1 {
            operation: Operation::Delegate {
                scenario: utils::DelegateScenario::Token(utils::TokenDelegateRole::Sale),
            }
            .to_string(),
            payload: payload.clone(),
            update_rule_state: false,
            rule_set_revision: None,
        })
        .unwrap()
        .instruction();

    // Fail to validate Transfer operation.
    let err =
        process_failing_validate_ix!(&mut context, validate_ix, vec![&additional_signer], None)
            .await;

    // Check that error is what we expect.
    assert_custom_error!(err, RuleSetError::AmountCheckFailed);

    // --------------------------------
    // Validate that when using second operation, we pass based on inputs that would fail the first
    // operation.
    // --------------------------------
    // Store a payload of data with an amount allowed by the the second operation's rule, but
    // ignored by the first operation's rule.
    let payload = Payload::from([(PayloadKey::Amount.to_string(), PayloadType::Number(2))]);

    // Create a `validate` instruction with the additional signer pubkey added but not sent as a
    // signer.  Send the second operation.
    let validate_ix = ValidateBuilder::new()
        .rule_set_pda(rule_set_addr)
        .mint(mint)
        .additional_rule_accounts(vec![AccountMeta::new_readonly(
            additional_signer.pubkey(),
            false,
        )])
        .build(ValidateArgs::V1 {
            operation: Operation::Delegate {
                scenario: utils::DelegateScenario::Token(utils::TokenDelegateRole::Sale),
            }
            .to_string(),
            payload: payload.clone(),
            update_rule_state: false,
            rule_set_revision: None,
        })
        .unwrap()
        .instruction();

    // Validate Transfer operation.
    process_passing_validate_ix!(&mut context, validate_ix, vec![], None).await;
}
