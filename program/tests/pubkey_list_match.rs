#![cfg(feature = "test-bpf")]

pub mod utils;

use mpl_token_auth_rules::{
    error::RuleSetError,
    instruction::{builders::ValidateBuilder, InstructionBuilder, ValidateArgs},
    payload::{Payload, PayloadKey, PayloadType},
    state::{Rule, RuleSet},
};
use solana_program_test::tokio;
use solana_sdk::{signature::Signer, signer::keypair::Keypair};
use utils::{
    assert_rule_set_error, create_rule_set_on_chain, process_failing_validate_ix,
    process_passing_validate_ix, program_test, Operation,
};

#[tokio::test]
async fn test_pubkey_list_match() {
    let mut context = program_test().start_with_context().await;

    // --------------------------------
    // Create RuleSet
    // --------------------------------
    // Create a Rule.
    let target_1 = Keypair::new();
    let target_2 = Keypair::new();
    let target_3 = Keypair::new();

    let rule = Rule::PubkeyListMatch {
        pubkeys: vec![target_1.pubkey(), target_2.pubkey(), target_3.pubkey()],
        field: PayloadKey::Target,
    };

    // Create a RuleSet.
    let mut rule_set = RuleSet::new("test rule_set".to_string(), context.payer.pubkey());
    rule_set.add(Operation::Transfer.to_string(), rule).unwrap();

    println!("{:#?}", rule_set);

    // Put the RuleSet on chain.
    let rule_set_addr =
        create_rule_set_on_chain(&mut context, rule_set, "test rule_set".to_string()).await;

    // --------------------------------
    // Validate fail
    // --------------------------------
    // Create a Keypair to simulate a token mint address.
    let mint = Keypair::new().pubkey();

    // Store the payload of data to validate against the rule definition with WRONG Pubkey.
    let payload = Payload::from([(
        PayloadKey::Target,
        PayloadType::Pubkey(Keypair::new().pubkey()),
    )]);

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

    // Fail to validate Transfer operation.
    let err = process_failing_validate_ix(&mut context, validate_ix, vec![]).await;

    // Check that error is what we expect.
    assert_rule_set_error(err, RuleSetError::PubkeyListMatchCheckFailed);

    // --------------------------------
    // Validate pass
    // --------------------------------
    // Create a Keypair to simulate a token mint address.
    let mint = Keypair::new().pubkey();

    // Store the payload of data to validate against the rule definition with CORRECT Pubkey.
    let payload = Payload::from([(PayloadKey::Target, PayloadType::Pubkey(target_2.pubkey()))]);

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

    // Validate Transfer operation.
    process_passing_validate_ix(&mut context, validate_ix, vec![]).await;
}