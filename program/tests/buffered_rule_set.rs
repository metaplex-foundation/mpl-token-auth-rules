#![cfg(feature = "test-bpf")]

pub mod utils;

use mpl_token_auth_rules::{
    error::RuleSetError,
    instruction::{builders::ValidateBuilder, InstructionBuilder, ValidateArgs},
    payload::{Payload, PayloadType},
    state::{Rule, RuleSetV1, RULE_SET_SERIALIZED_HEADER_LEN},
};
use rmp_serde::Serializer;
use serde::Serialize;
use solana_program::system_program;
use solana_program_test::tokio;
use solana_sdk::{signature::Signer, signer::keypair::Keypair};
use utils::{cmp_slice, program_test, Operation, PayloadKey};

#[tokio::test]
async fn buffered_rule_set() {
    let mut context = program_test().start_with_context().await;

    // --------------------------------
    // Create RuleSet
    // --------------------------------
    // Create a Rule.
    let targets = (0..70).map(|_| system_program::ID).collect::<Vec<_>>();

    let rule = Rule::PubkeyListMatch {
        pubkeys: targets,
        field: PayloadKey::Authority.to_string(),
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
    let test_rule_set = rule_set.clone();

    // Put the RuleSet on chain.
    let rule_set_addr =
        create_big_rule_set_on_chain!(&mut context, rule_set, "test rule_set".to_string(), None)
            .await;

    // Serialize the RuleSet using RMP serde.
    let mut serialized_rule_set = Vec::new();
    test_rule_set
        .serialize(&mut Serializer::new(&mut serialized_rule_set))
        .unwrap();

    let data = context
        .banks_client
        .get_account(rule_set_addr)
        .await
        .unwrap()
        .unwrap()
        .data;

    // Because there is only one RuleSet we can assume it exists right after the header.
    // TODO: Write utility function to provide the RuleSet location of a given revision.
    let start = RULE_SET_SERIALIZED_HEADER_LEN + 1;
    let end = RULE_SET_SERIALIZED_HEADER_LEN + 1 + serialized_rule_set.len();
    assert!(
        cmp_slice(&data[start..end], &serialized_rule_set),
        "The buffer doesn't match the serialized rule set.",
    );

    // --------------------------------
    // Validate fail
    // --------------------------------
    // Create a Keypair to simulate a token mint address.
    let mint = Keypair::new().pubkey();

    // Store the payload of data to validate against the rule definition with WRONG Pubkey.
    let payload = Payload::from([(
        PayloadKey::Authority.to_string(),
        PayloadType::Pubkey(Keypair::new().pubkey()),
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
    assert_custom_error!(err, RuleSetError::PubkeyListMatchCheckFailed);

    // --------------------------------
    // Validate pass
    // --------------------------------
    // Create a Keypair to simulate a token mint address.
    let mint = Keypair::new().pubkey();

    // Store the payload of data to validate against the rule definition with CORRECT Pubkey.
    let payload = Payload::from([(
        PayloadKey::Authority.to_string(),
        PayloadType::Pubkey(system_program::ID),
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

    // Validate Transfer operation.
    process_passing_validate_ix!(&mut context, validate_ix, vec![], None).await;
}
