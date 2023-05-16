#![cfg(feature = "test-bpf")]

pub mod utils;

use mpl_token_auth_rules::{
    error::RuleSetError,
    instruction::{builders::ValidateBuilder, InstructionBuilder, ValidateArgs},
    payload::Payload,
    state::{Rule, RuleSetV1},
};
use solana_program::{instruction::AccountMeta, pubkey::Pubkey};
use solana_program_test::{tokio, ProgramTestContext};
use solana_sdk::{signature::Signer, signer::keypair::Keypair};
use utils::{program_test, Operation};

async fn create_rule_set(context: &mut ProgramTestContext) -> (Pubkey, Keypair) {
    // --------------------------------
    // Create RuleSet
    // --------------------------------
    // Create some rules.
    let adtl_signer = Keypair::new();
    let adtl_signer_rule = Rule::AdditionalSigner {
        account: adtl_signer.pubkey(),
    };

    // Create a RuleSet.
    let mut rule_set = RuleSetV1::new("test rule_set".to_string(), context.payer.pubkey());
    rule_set
        .add(
            Operation::Transfer {
                scenario: utils::TransferScenario::Holder,
            }
            .to_string(),
            adtl_signer_rule,
        )
        .unwrap();

    println!("{:#?}", rule_set);

    // Put the RuleSet on chain.
    let rule_set_addr =
        create_rule_set_on_chain!(context, rule_set, "test rule_set".to_string()).await;

    (rule_set_addr, adtl_signer)
}

async fn create_not_rule_set(context: &mut ProgramTestContext) -> (Pubkey, Keypair) {
    // --------------------------------
    // Create RuleSet
    // --------------------------------
    // Create some rules.
    let adtl_signer = Keypair::new();
    let adtl_signer_rule = Rule::Not {
        rule: Box::new(Rule::AdditionalSigner {
            account: adtl_signer.pubkey(),
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
            adtl_signer_rule,
        )
        .unwrap();

    // Put the RuleSet on chain.
    let rule_set_addr =
        create_rule_set_on_chain!(context, rule_set, "test rule_set".to_string()).await;

    (rule_set_addr, adtl_signer)
}

#[tokio::test]
async fn test_additional_signer_missing_account() {
    let mut context = program_test().start_with_context().await;

    let (rule_set_addr, _) = create_rule_set(&mut context).await;
    let mint = Keypair::new().pubkey();

    // --------------------------------
    // Validate fail missing account
    // --------------------------------
    // Create a Keypair to simulate a token mint address.

    // Create a `validate` instruction WITHOUT the additional signer.
    let validate_ix = ValidateBuilder::new()
        .rule_set_pda(rule_set_addr)
        .mint(mint)
        .additional_rule_accounts(vec![])
        .build(ValidateArgs::V1 {
            operation: Operation::Transfer {
                scenario: utils::TransferScenario::Holder,
            }
            .to_string(),
            payload: Payload::default(),
            update_rule_state: false,
            rule_set_revision: None,
        })
        .unwrap()
        .instruction();

    // Fail to validate Transfer operation.
    let err = process_failing_validate_ix!(&mut context, validate_ix, vec![], None).await;

    // Check that error is what we expect.
    assert_custom_error!(err, RuleSetError::MissingAccount);
}

#[tokio::test]
async fn test_not_additional_signer_missing_account() {
    let mut context = program_test().start_with_context().await;

    let (rule_set_addr, _) = create_not_rule_set(&mut context).await;
    let mint = Keypair::new().pubkey();

    // --------------------------------
    // Validate fail missing account
    // --------------------------------
    // Create a Keypair to simulate a token mint address.

    // Create a `validate` instruction WITHOUT the additional signer.
    let validate_ix = ValidateBuilder::new()
        .rule_set_pda(rule_set_addr)
        .mint(mint)
        .additional_rule_accounts(vec![])
        .build(ValidateArgs::V1 {
            operation: Operation::Transfer {
                scenario: utils::TransferScenario::Holder,
            }
            .to_string(),
            payload: Payload::default(),
            update_rule_state: false,
            rule_set_revision: None,
        })
        .unwrap()
        .instruction();

    // Fail to validate Transfer operation.
    let err = process_failing_validate_ix!(&mut context, validate_ix, vec![], None).await;

    // Check that error is what we expect.
    assert_custom_error!(err, RuleSetError::MissingAccount);
}

#[tokio::test]
async fn test_additional_signer_not_signer() {
    let mut context = program_test().start_with_context().await;

    let (rule_set_addr, adtl_signer) = create_rule_set(&mut context).await;
    let mint = Keypair::new().pubkey();

    // --------------------------------
    // Validate fail not a signer
    // --------------------------------

    // Create a `validate` instruction WITH the additional account but not as a signer.
    let validate_ix = ValidateBuilder::new()
        .rule_set_pda(rule_set_addr)
        .mint(mint)
        .additional_rule_accounts(vec![AccountMeta::new_readonly(adtl_signer.pubkey(), false)])
        .build(ValidateArgs::V1 {
            operation: Operation::Transfer {
                scenario: utils::TransferScenario::Holder,
            }
            .to_string(),
            payload: Payload::default(),
            update_rule_state: false,
            rule_set_revision: None,
        })
        .unwrap()
        .instruction();

    // Validate Transfer operation.
    let err = process_failing_validate_ix!(&mut context, validate_ix, vec![], None).await;

    // Check that error is what we expect.
    assert_custom_error!(err, RuleSetError::AdditionalSignerCheckFailed);
}

#[tokio::test]
async fn test_not_additional_signer_not_signer() {
    let mut context = program_test().start_with_context().await;

    let (rule_set_addr, adtl_signer) = create_not_rule_set(&mut context).await;
    let mint = Keypair::new().pubkey();

    // --------------------------------
    // Validate passes when not a signer
    // --------------------------------

    // Create a `validate` instruction WITH the additional account but not as a signer.
    let validate_ix = ValidateBuilder::new()
        .rule_set_pda(rule_set_addr)
        .mint(mint)
        .additional_rule_accounts(vec![AccountMeta::new_readonly(adtl_signer.pubkey(), false)])
        .build(ValidateArgs::V1 {
            operation: Operation::Transfer {
                scenario: utils::TransferScenario::Holder,
            }
            .to_string(),
            payload: Payload::default(),
            update_rule_state: false,
            rule_set_revision: None,
        })
        .unwrap()
        .instruction();

    // Validate Transfer operation.
    process_passing_validate_ix!(&mut context, validate_ix, vec![], None).await;
}

#[tokio::test]
async fn test_additional_signer_pass() {
    let mut context = program_test().start_with_context().await;

    let (rule_set_addr, adtl_signer) = create_rule_set(&mut context).await;
    let mint = Keypair::new().pubkey();

    // --------------------------------
    // Validate pass
    // --------------------------------
    // Create a `validate` instruction WITH the additional signer.
    let validate_ix = ValidateBuilder::new()
        .rule_set_pda(rule_set_addr)
        .mint(mint)
        .additional_rule_accounts(vec![AccountMeta::new_readonly(adtl_signer.pubkey(), true)])
        .build(ValidateArgs::V1 {
            operation: Operation::Transfer {
                scenario: utils::TransferScenario::Holder,
            }
            .to_string(),
            payload: Payload::default(),
            update_rule_state: false,
            rule_set_revision: None,
        })
        .unwrap()
        .instruction();

    // Validate Transfer operation.
    process_passing_validate_ix!(&mut context, validate_ix, vec![&adtl_signer], None).await;
}

#[tokio::test]
async fn test_not_additional_signer_fail() {
    let mut context = program_test().start_with_context().await;

    let (rule_set_addr, adtl_signer) = create_not_rule_set(&mut context).await;
    let mint = Keypair::new().pubkey();

    // --------------------------------
    // Validate fail
    // --------------------------------
    // Create a `validate` instruction WITH the additional signer.
    let validate_ix = ValidateBuilder::new()
        .rule_set_pda(rule_set_addr)
        .mint(mint)
        .additional_rule_accounts(vec![AccountMeta::new_readonly(adtl_signer.pubkey(), true)])
        .build(ValidateArgs::V1 {
            operation: Operation::Transfer {
                scenario: utils::TransferScenario::Holder,
            }
            .to_string(),
            payload: Payload::default(),
            update_rule_state: false,
            rule_set_revision: None,
        })
        .unwrap()
        .instruction();

    // Validate Transfer operation.
    let err =
        process_failing_validate_ix!(&mut context, validate_ix, vec![&adtl_signer], None).await;

    // Check that error is what we expect.
    assert_custom_error!(err, RuleSetError::AdditionalSignerCheckFailed);
}
