#![cfg(feature = "test-bpf")]

pub mod utils;

use mpl_token_auth_rules::{
    error::RuleSetError,
    instruction::{builders::ValidateBuilder, InstructionBuilder, ValidateArgs},
    payload::Payload,
    pda::find_rule_set_state_address,
    state::{Rule, RuleSet},
};
use solana_program::program_error::ProgramError;
use solana_program_test::tokio;
use solana_sdk::{signature::Signer, signer::keypair::Keypair};
use utils::{
    assert_program_error, assert_rule_set_error, create_rule_set_on_chain,
    process_failing_validate_ix, program_test, Operation,
};

#[tokio::test]
async fn test_frequency() {
    let mut context = program_test().start_with_context().await;

    // --------------------------------
    // Create RuleSet
    // --------------------------------
    // Create a Rule.
    let rule_authority = Keypair::new();
    let rule = Rule::Frequency {
        authority: rule_authority.pubkey(),
    };

    // Create a RuleSet.
    let mut rule_set = RuleSet::new("test rule_set".to_string(), context.payer.pubkey());
    rule_set
        .add(Operation::OwnerTransfer.to_string(), rule)
        .unwrap();

    println!("{:#?}", rule_set);

    // Put the RuleSet on chain.
    let rule_set_addr =
        create_rule_set_on_chain(&mut context, rule_set, "test rule_set".to_string()).await;

    // --------------------------------
    // Validate missing accounts
    // --------------------------------
    // Create a Keypair to simulate a token mint address.
    let mint = Keypair::new().pubkey();

    // Create a `validate` instruction.
    let validate_ix = ValidateBuilder::new()
        .rule_set_pda(rule_set_addr)
        .mint(mint)
        .additional_rule_accounts(vec![])
        .build(ValidateArgs::V1 {
            operation: Operation::OwnerTransfer.to_string(),
            payload: Payload::default(),
            update_rule_state: true,
        })
        .unwrap()
        .instruction();

    // Fail to validate Transfer operation.
    let err = process_failing_validate_ix(&mut context, validate_ix, vec![]).await;

    // Check that error is what we expect.
    assert_program_error(err, ProgramError::NotEnoughAccountKeys);

    // --------------------------------
    // Validate wrong authority
    // --------------------------------
    let (rule_set_state_addr, _rule_set_bump) =
        find_rule_set_state_address(context.payer.pubkey(), "test rule_set".to_string(), mint);

    // Create a `validate` instruction.
    let validate_ix = ValidateBuilder::new()
        .rule_set_pda(rule_set_addr)
        .mint(mint)
        .payer(context.payer.pubkey())
        .rule_authority(context.payer.pubkey())
        .rule_set_state_pda(rule_set_state_addr)
        .additional_rule_accounts(vec![])
        .build(ValidateArgs::V1 {
            operation: Operation::OwnerTransfer.to_string(),
            payload: Payload::default(),
            update_rule_state: true,
        })
        .unwrap()
        .instruction();

    // Fail to validate Transfer operation.
    let err = process_failing_validate_ix(&mut context, validate_ix, vec![]).await;

    // Check that error is what we expect.
    assert_rule_set_error(err, RuleSetError::RuleAuthorityIsNotSigner);

    // --------------------------------
    // Validate not implemented
    // (this will become pass later)
    // --------------------------------
    // Create a `validate` instruction.
    let validate_ix = ValidateBuilder::new()
        .rule_set_pda(rule_set_addr)
        .mint(mint)
        .payer(context.payer.pubkey())
        .rule_authority(rule_authority.pubkey())
        .rule_set_state_pda(rule_set_state_addr)
        .additional_rule_accounts(vec![])
        .build(ValidateArgs::V1 {
            operation: Operation::OwnerTransfer.to_string(),
            payload: Payload::default(),
            update_rule_state: true,
        })
        .unwrap()
        .instruction();

    // Fail to validate Transfer operation.
    let err = process_failing_validate_ix(&mut context, validate_ix, vec![&rule_authority]).await;

    // Check that error is what we expect.
    assert_rule_set_error(err, RuleSetError::NotImplemented);
}
