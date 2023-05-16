#![cfg(feature = "test-bpf")]

pub mod utils;

use mpl_token_auth_rules::{
    error::RuleSetError,
    instruction::{builders::ValidateBuilder, InstructionBuilder, ValidateArgs},
    payload::{Payload, PayloadType},
    state::{CompareOp, Rule, RuleSetV1},
};
use solana_program::{instruction::AccountMeta, pubkey::Pubkey};
use solana_program_test::{tokio, ProgramTestContext};
use solana_sdk::{signature::Signer, signer::keypair::Keypair};
use utils::{program_test, Operation, PayloadKey};

async fn create_rule_set(context: &mut ProgramTestContext) -> (Pubkey, Keypair) {
    // --------------------------------
    // Create RuleSet
    // --------------------------------
    // Create some rules.
    let second_signer = Keypair::new();
    let adtl_signer = Rule::AdditionalSigner {
        account: second_signer.pubkey(),
    };

    let amount_check = Rule::Amount {
        amount: 5,
        operator: CompareOp::Lt,
        field: PayloadKey::Amount.to_string(),
    };

    let overall_rule = Rule::All {
        rules: vec![adtl_signer, amount_check],
    };

    // Create a RuleSet.
    let mut rule_set = RuleSetV1::new("test rule_set".to_string(), context.payer.pubkey());
    rule_set
        .add(
            Operation::Transfer {
                scenario: utils::TransferScenario::Holder,
            }
            .to_string(),
            overall_rule,
        )
        .unwrap();

    println!("{:#?}", rule_set);

    // Put the RuleSet on chain.
    let rule_set_addr =
        create_rule_set_on_chain!(context, rule_set, "test rule_set".to_string()).await;

    (rule_set_addr, second_signer)
}

async fn create_not_rule_set(context: &mut ProgramTestContext) -> (Pubkey, Keypair) {
    // --------------------------------
    // Create RuleSet
    // --------------------------------
    // Create some rules.
    let second_signer = Keypair::new();
    let adtl_signer = Rule::AdditionalSigner {
        account: second_signer.pubkey(),
    };

    let amount_check = Rule::Amount {
        amount: 5,
        operator: CompareOp::Lt,
        field: PayloadKey::Amount.to_string(),
    };

    let overall_rule = Rule::Not {
        rule: Box::new(Rule::All {
            rules: vec![adtl_signer, amount_check],
        }),
    };

    // Create a RuleSet.
    let mut rule_set = RuleSetV1::new("test rule_set".to_string(), context.payer.pubkey());
    rule_set
        .add(
            Operation::Transfer {
                scenario: utils::TransferScenario::Holder,
            }
            .to_string(),
            overall_rule,
        )
        .unwrap();

    // Put the RuleSet on chain.
    let rule_set_addr =
        create_rule_set_on_chain!(context, rule_set, "test rule_set".to_string()).await;

    (rule_set_addr, second_signer)
}

#[tokio::test]
async fn test_all_fail() {
    let mut context = program_test().start_with_context().await;

    let (rule_set_addr, second_signer) = create_rule_set(&mut context).await;

    // Create a Keypair to simulate a token mint address.
    let mint = Keypair::new().pubkey();

    // --------------------------------
    // Validate fail
    // --------------------------------

    // Store a payload of data with the WRONG amount.
    let payload = Payload::from([(PayloadKey::Amount.to_string(), PayloadType::Number(5))]);

    // Create a `validate` instruction with the additional signer but sending WRONG amount.
    let validate_ix = ValidateBuilder::new()
        .rule_set_pda(rule_set_addr)
        .mint(mint)
        .additional_rule_accounts(vec![AccountMeta::new_readonly(
            second_signer.pubkey(),
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

    // Fail to validate Transfer operation.
    let err =
        process_failing_validate_ix!(&mut context, validate_ix, vec![&second_signer], None).await;

    // Check that error is what we expect.  In this case we expect the first failure to roll up.
    assert_custom_error!(err, RuleSetError::AmountCheckFailed);
}

#[tokio::test]
async fn test_all_success() {
    let mut context = program_test().start_with_context().await;

    let (rule_set_addr, second_signer) = create_rule_set(&mut context).await;

    // Create a Keypair to simulate a token mint address.
    let mint = Keypair::new().pubkey();

    // --------------------------------
    // Validate pass
    // --------------------------------
    // Store a payload of data with the CORRECT amount.
    let payload = Payload::from([(PayloadKey::Amount.to_string(), PayloadType::Number(4))]);

    // Create a `validate` instruction with the additional signer AND sending CORRECT amount.
    let validate_ix = ValidateBuilder::new()
        .rule_set_pda(rule_set_addr)
        .mint(mint)
        .additional_rule_accounts(vec![AccountMeta::new_readonly(
            second_signer.pubkey(),
            true,
        )])
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

    // Validate Transfer operation since both Rule conditions were true.
    process_passing_validate_ix!(&mut context, validate_ix, vec![&second_signer], None).await;
}

#[tokio::test]
async fn test_not_all_success() {
    let mut context = program_test().start_with_context().await;

    let (rule_set_addr, second_signer) = create_not_rule_set(&mut context).await;

    // Create a Keypair to simulate a token mint address.
    let mint = Keypair::new().pubkey();

    // --------------------------------
    // Validate pass
    // --------------------------------

    // Store a payload of data with the WRONG amount.
    let payload = Payload::from([(PayloadKey::Amount.to_string(), PayloadType::Number(5))]);

    // Create a `validate` instruction with the additional signer but sending WRONG amount.
    let validate_ix = ValidateBuilder::new()
        .rule_set_pda(rule_set_addr)
        .mint(mint)
        .additional_rule_accounts(vec![AccountMeta::new_readonly(
            second_signer.pubkey(),
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

    // Fail to validate Transfer operation.
    process_passing_validate_ix!(&mut context, validate_ix, vec![&second_signer], None).await;
}

#[tokio::test]
async fn test_not_all_fail() {
    let mut context = program_test().start_with_context().await;

    let (rule_set_addr, second_signer) = create_not_rule_set(&mut context).await;

    // Create a Keypair to simulate a token mint address.
    let mint = Keypair::new().pubkey();

    // --------------------------------
    // Validate fail
    // --------------------------------
    // Store a payload of data with the CORRECT amount.
    let payload = Payload::from([(PayloadKey::Amount.to_string(), PayloadType::Number(4))]);

    // Create a `validate` instruction with the additional signer AND sending CORRECT amount.
    let validate_ix = ValidateBuilder::new()
        .rule_set_pda(rule_set_addr)
        .mint(mint)
        .additional_rule_accounts(vec![AccountMeta::new_readonly(
            second_signer.pubkey(),
            true,
        )])
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

    // Validate Transfer operation since both Rule conditions were true.
    let err =
        process_failing_validate_ix!(&mut context, validate_ix, vec![&second_signer], None).await;

    assert_custom_error!(err, RuleSetError::AmountCheckFailed);
}
