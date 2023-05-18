#![cfg(feature = "test-bpf")]

pub mod utils;

use mpl_token_auth_rules::{
    error::RuleSetError,
    instruction::{builders::ValidateBuilder, InstructionBuilder, ValidateArgs},
    payload::{Payload, PayloadType, SeedsVec},
    state::{Rule, RuleSetV1},
};
use solana_program::{instruction::AccountMeta, pubkey::Pubkey};
use solana_program_test::tokio;
use solana_sdk::{signature::Signer, signer::keypair::Keypair};
use utils::{program_test, Operation, PayloadKey};

#[tokio::test]
async fn test_pda_match_assumed_owner() {
    let mut context = program_test().start_with_context().await;

    // --------------------------------
    // Create RuleSet
    // --------------------------------
    // Create a Rule.
    let rule = Rule::PDAMatch {
        program: None,
        pda_field: PayloadKey::Destination.to_string(),
        seeds_field: PayloadKey::DestinationSeeds.to_string(),
    };

    // Create a RuleSet.
    let mut rule_set = RuleSetV1::new("test rule_set".to_string(), context.payer.pubkey());
    rule_set
        .add(
            Operation::Transfer {
                scenario: utils::TransferScenario::Holder,
            }
            .to_string(),
            rule,
        )
        .unwrap();

    println!("{:#?}", rule_set);

    // Put the RuleSet on chain.
    let rule_set_addr =
        create_rule_set_on_chain!(&mut context, rule_set, "test rule_set".to_string()).await;

    // --------------------------------
    // Validate fail
    // --------------------------------
    // Create a Keypair to simulate a token mint address.
    let mint = Keypair::new().pubkey();

    // Our derived key is going to be an account owned by the
    // mpl-token-auth-rules program. Any one will do so for convenience
    // we just use the RuleSet.  These are the RuleSet seeds.
    let seeds = vec![
        mpl_token_auth_rules::pda::PREFIX.as_bytes().to_vec(),
        context.payer.pubkey().as_ref().to_vec(),
        "test rule_set".as_bytes().to_vec(),
    ];

    // Store the payload of data to validate against the rule definition, using an invalid PDA.
    let invalid_pda = Keypair::new().pubkey();
    let payload = Payload::from([
        (
            PayloadKey::Destination.to_string(),
            PayloadType::Pubkey(invalid_pda),
        ),
        (
            PayloadKey::DestinationSeeds.to_string(),
            PayloadType::Seeds(SeedsVec::new(seeds.clone())),
        ),
    ]);

    // Create a `validate` instruction.
    let validate_ix = ValidateBuilder::new()
        .rule_set_pda(rule_set_addr)
        .mint(mint)
        .additional_rule_accounts(vec![AccountMeta::new_readonly(invalid_pda, false)])
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
    assert_custom_error!(err, RuleSetError::PDAMatchCheckFailed);

    // --------------------------------
    // Validate pass
    // --------------------------------
    // Store the payload of data to validate against the rule definition, using a correct PDA.
    let payload = Payload::from([
        (
            PayloadKey::Destination.to_string(),
            PayloadType::Pubkey(rule_set_addr),
        ),
        (
            PayloadKey::DestinationSeeds.to_string(),
            PayloadType::Seeds(SeedsVec::new(seeds)),
        ),
    ]);

    // Create a `validate` instruction.
    let validate_ix = ValidateBuilder::new()
        .rule_set_pda(rule_set_addr)
        .mint(mint)
        .additional_rule_accounts(vec![AccountMeta::new_readonly(rule_set_addr, false)])
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

    // Validate Transfer operation.
    process_passing_validate_ix!(&mut context, validate_ix, vec![], None).await;
}

#[tokio::test]
async fn test_pda_match_specified_owner() {
    let mut context = program_test().start_with_context().await;

    // --------------------------------
    // Create RuleSet
    // --------------------------------
    // Create a Rule.
    let rule = Rule::PDAMatch {
        program: Some(mpl_token_auth_rules::ID),
        pda_field: PayloadKey::Authority.to_string(),
        seeds_field: PayloadKey::AuthoritySeeds.to_string(),
    };

    // Create a RuleSet.
    let mut rule_set = RuleSetV1::new("test rule_set".to_string(), context.payer.pubkey());
    rule_set
        .add(
            Operation::Transfer {
                scenario: utils::TransferScenario::Holder,
            }
            .to_string(),
            rule,
        )
        .unwrap();

    println!("{:#?}", rule_set);

    // Put the RuleSet on chain.
    let rule_set_addr =
        create_rule_set_on_chain!(&mut context, rule_set, "test rule_set".to_string()).await;

    // --------------------------------
    // Validate fail
    // --------------------------------
    // Create a Keypair to simulate a token mint address.
    let mint = Keypair::new().pubkey();

    let seeds = vec!["Hello".as_bytes().to_vec(), mint.as_ref().to_vec()];

    // Store the payload of data to validate against the rule definition, using an invalid PDA.
    let invalid_pda = Keypair::new().pubkey();
    let payload = Payload::from([
        (
            PayloadKey::Authority.to_string(),
            PayloadType::Pubkey(invalid_pda),
        ),
        (
            PayloadKey::AuthoritySeeds.to_string(),
            PayloadType::Seeds(SeedsVec::new(seeds.clone())),
        ),
    ]);

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
            payload: payload.clone(),
            update_rule_state: false,
            rule_set_revision: None,
        })
        .unwrap()
        .instruction();

    // Fail to validate Transfer operation.
    let err = process_failing_validate_ix!(&mut context, validate_ix, vec![], None).await;

    // Check that error is what we expect.
    assert_custom_error!(err, RuleSetError::PDAMatchCheckFailed);

    // --------------------------------
    // Validate pass
    // --------------------------------
    // Store the payload of data to validate against the rule definition, using a correct PDA.
    let vec_of_slices = seeds.iter().map(Vec::as_slice).collect::<Vec<&[u8]>>();
    let (valid_pda, _bump) =
        Pubkey::find_program_address(&vec_of_slices, &mpl_token_auth_rules::ID);

    let payload = Payload::from([
        (
            PayloadKey::Authority.to_string(),
            PayloadType::Pubkey(valid_pda),
        ),
        (
            PayloadKey::AuthoritySeeds.to_string(),
            PayloadType::Seeds(SeedsVec::new(seeds)),
        ),
    ]);

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

    // Validate Transfer operation.
    process_passing_validate_ix!(&mut context, validate_ix, vec![], None).await;
}
