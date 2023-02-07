#![cfg(feature = "test-bpf")]

pub mod utils;

use borsh::BorshSerialize;
use mpl_token_auth_rules::{
    error::RuleSetError,
    instruction::{builders::ValidateBuilder, InstructionBuilder, ValidateArgs},
    payload::{Payload, PayloadType},
    state::{
        CompareOp, Rule, RuleSetHeader, RuleSetRevisionMapV1, RuleSetV1, RULE_SET_LIB_VERSION,
        RULE_SET_REV_MAP_VERSION, RULE_SET_SERIALIZED_HEADER_LEN,
    },
};
use rmp_serde::Serializer;
use serde::Serialize;
use solana_program::instruction::AccountMeta;
use solana_program_test::tokio;
use solana_sdk::{signature::Signer, signer::keypair::Keypair};
use utils::{cmp_slice, program_test, Operation, PayloadKey};

#[tokio::test]
async fn test_update_ruleset_data_integrity() {
    let mut context = program_test().start_with_context().await;

    // --------------------------------
    // Create RuleSet 0
    // --------------------------------
    let mut rule_sets = vec![];

    // Create a Pass Rule as the overall rule.
    let first_overall_rule = Rule::Pass;

    // Create a RuleSet.
    let mut rule_set = RuleSetV1::new("test rule_set".to_string(), context.payer.pubkey());
    rule_set
        .add(
            Operation::Transfer {
                scenario: utils::TransferScenario::Holder,
            }
            .to_string(),
            first_overall_rule.clone(),
        )
        .unwrap();

    // Save RuleSet for validation later.
    rule_sets.push(rule_set.clone());

    // Put the RuleSet on chain.
    let _rule_set_addr =
        create_rule_set_on_chain!(&mut context, rule_set, "test rule_set".to_string()).await;

    // Move forward to avoid duplicate transactions if RuleSets are same.
    let mut slot = 3;
    context.warp_to_slot(slot).unwrap();
    slot += 1;

    // --------------------------------
    // Create RuleSet 1 and update on chain
    // --------------------------------
    // Create some other rules.
    let adtl_signer = Rule::AdditionalSigner {
        account: context.payer.pubkey(),
    };

    let amount_check = Rule::Amount {
        amount: 1,
        operator: CompareOp::Eq,
        field: PayloadKey::Amount.to_string(),
    };

    let second_overall_rule = Rule::All {
        rules: vec![adtl_signer, amount_check],
    };

    // Create a new RuleSet.
    let mut rule_set = RuleSetV1::new("test rule_set".to_string(), context.payer.pubkey());
    rule_set
        .add(
            Operation::Delegate {
                scenario: utils::DelegateScenario::Token(utils::TokenDelegateRole::Sale),
            }
            .to_string(),
            second_overall_rule.clone(),
        )
        .unwrap();

    rule_set
        .add(
            Operation::Transfer {
                scenario: utils::TransferScenario::SaleDelegate,
            }
            .to_string(),
            second_overall_rule.clone(),
        )
        .unwrap();

    // Save RuleSet for validation later.
    rule_sets.push(rule_set.clone());

    // Put the updated RuleSet on chain.
    let _rule_set_addr =
        create_rule_set_on_chain!(&mut context, rule_set, "test rule_set".to_string()).await;

    // Move forward to avoid duplicate transactions if RuleSets are same.
    context.warp_to_slot(slot).unwrap();
    slot += 1;

    // --------------------------------
    // Create RuleSet 2 and update on chain
    // --------------------------------
    let program_owned = Rule::ProgramOwned {
        program: mpl_token_auth_rules::ID,
        field: PayloadKey::Destination.to_string(),
    };

    let target_1 = Keypair::new();
    let target_2 = Keypair::new();
    let target_3 = Keypair::new();

    let list_match = Rule::PubkeyListMatch {
        pubkeys: vec![target_1.pubkey(), target_2.pubkey(), target_3.pubkey()],
        field: PayloadKey::Authority.to_string(),
    };

    let third_overall_rule = Rule::Any {
        rules: vec![program_owned, list_match],
    };

    // Create a new RuleSet.
    let mut rule_set = RuleSetV1::new("test rule_set".to_string(), context.payer.pubkey());
    rule_set
        .add(
            Operation::Transfer {
                scenario: utils::TransferScenario::Holder,
            }
            .to_string(),
            third_overall_rule.clone(),
        )
        .unwrap();

    // Save RuleSet for validation later.
    rule_sets.push(rule_set.clone());

    // Put the updated RuleSet on chain.
    let _rule_set_addr =
        create_rule_set_on_chain!(&mut context, rule_set, "test rule_set".to_string()).await;

    // Move forward to avoid duplicate transactions if RuleSets are same.
    context.warp_to_slot(slot).unwrap();
    slot += 1;

    // --------------------------------
    // Create RuleSet 3 and update on chain
    // --------------------------------
    // Create a new RuleSet reusing some previous rules.
    let mut rule_set = RuleSetV1::new("test rule_set".to_string(), context.payer.pubkey());
    rule_set
        .add(
            Operation::Transfer {
                scenario: utils::TransferScenario::Holder,
            }
            .to_string(),
            first_overall_rule,
        )
        .unwrap();
    rule_set
        .add(
            Operation::Delegate {
                scenario: utils::DelegateScenario::Token(utils::TokenDelegateRole::Sale),
            }
            .to_string(),
            second_overall_rule.clone(),
        )
        .unwrap();
    rule_set
        .add(
            Operation::Transfer {
                scenario: utils::TransferScenario::TransferDelegate,
            }
            .to_string(),
            second_overall_rule,
        )
        .unwrap();

    // Save RuleSet for validation later.
    rule_sets.push(rule_set.clone());

    // Put the updated RuleSet on chain.
    let _rule_set_addr =
        create_rule_set_on_chain!(&mut context, rule_set, "test rule_set".to_string()).await;

    // Move forward to avoid duplicate transactions if RuleSets are same.
    context.warp_to_slot(slot).unwrap();
    slot += 1;

    // --------------------------------
    // Create RuleSet 4 and update on chain
    // --------------------------------
    // Create a new RuleSet reusing some previous rules.
    let mut rule_set = RuleSetV1::new("test rule_set".to_string(), context.payer.pubkey());
    rule_set
        .add(
            Operation::Transfer {
                scenario: utils::TransferScenario::Holder,
            }
            .to_string(),
            third_overall_rule,
        )
        .unwrap();

    // Save RuleSet for validation later.
    rule_sets.push(rule_set.clone());

    // Put the updated RuleSet on chain.
    let rule_set_addr =
        create_rule_set_on_chain!(&mut context, rule_set, "test rule_set".to_string()).await;

    // Move forward to avoid duplicate transactions if RuleSets are same.
    context.warp_to_slot(slot).unwrap();

    // --------------------------------
    // Validate the on chain data for all RuleSets
    // --------------------------------
    // Get the `RuleSet` PDA data.
    let data = context
        .banks_client
        .get_account(rule_set_addr)
        .await
        .unwrap()
        .unwrap()
        .data;

    // Check all the RuleSets, saving their start locations for later use.
    let mut offsets = vec![RULE_SET_SERIALIZED_HEADER_LEN];
    for n in 0..rule_sets.len() {
        // Offset n is the `RuleSet` lib version location.
        let rule_set_version_loc = offsets[n];

        // Check the nth `RuleSet` lib version.
        assert_eq!(
            data[rule_set_version_loc], RULE_SET_LIB_VERSION,
            "The buffer doesn't match rule set {} lib version,",
            n
        );

        // Serialize the nth `RuleSet` using RMP serde.
        let mut serialized_rule_set = Vec::new();
        rule_sets[n]
            .serialize(&mut Serializer::new(&mut serialized_rule_set))
            .unwrap();

        // Check the first `RuleSet` serialized data.
        let rule_set_start = rule_set_version_loc + 1;
        let rule_set_end = rule_set_start + serialized_rule_set.len();
        assert!(
            cmp_slice(&data[rule_set_start..rule_set_end], &serialized_rule_set),
            "The buffer doesn't match the serialized rule set {}.",
            n,
        );

        // The end of `RuleSet` n is the offset for the next item.
        offsets.push(rule_set_end)
    }

    // The final offset is the end of the last `RuleSet` and thus the start of the revision map.
    let rev_map_version_loc = *offsets.last().unwrap();

    // Check the revision map version.
    assert_eq!(
        data[rev_map_version_loc], RULE_SET_REV_MAP_VERSION,
        "The buffer doesn't match the revision map version"
    );

    // Create revision map using the known locations of the two `RuleSet`s in this test.
    let mut revision_map = RuleSetRevisionMapV1::default();

    // Push the `RuleSet` locations.
    for loc in offsets.iter().take(rule_sets.len()) {
        revision_map.rule_set_revisions.push(*loc);
    }

    // Borsh serialize the revision map.
    let mut serialized_rev_map = Vec::new();
    revision_map.serialize(&mut serialized_rev_map).unwrap();

    // Check the revision map.  This should go to the end of the data slice.
    let rev_map_start = rev_map_version_loc + 1;
    assert!(
        cmp_slice(&data[rev_map_start..], &serialized_rev_map),
        "The buffer doesn't match the serialized revision map.",
    );

    // Create header using the known location of the revision map version location.
    let header = RuleSetHeader::new(rev_map_version_loc);

    // Borsh serialize the header.
    let mut serialized_header = Vec::new();
    header.serialize(&mut serialized_header).unwrap();

    // Check the header.
    assert!(
        cmp_slice(&data[..RULE_SET_SERIALIZED_HEADER_LEN], &serialized_header),
        "The buffer doesn't match the serialized header.",
    );
}

#[tokio::test]
async fn test_unknown_rule_set_revision_fails() {
    let mut context = program_test().start_with_context().await;

    // --------------------------------
    // Create RuleSets
    // --------------------------------
    let additional_signer = Keypair::new();
    let adtl_signer_rule = Rule::AdditionalSigner {
        account: additional_signer.pubkey(),
    };

    // Create a RuleSet.
    let mut first_rule_set = RuleSetV1::new("test rule_set".to_string(), context.payer.pubkey());
    first_rule_set
        .add(
            Operation::Transfer {
                scenario: utils::TransferScenario::Holder,
            }
            .to_string(),
            adtl_signer_rule,
        )
        .unwrap();

    // Put the RuleSet on chain.
    let _rule_set_addr = create_rule_set_on_chain!(
        &mut context,
        first_rule_set.clone(),
        "test rule_set".to_string()
    )
    .await;

    let amount_check = Rule::Amount {
        amount: 10,
        operator: CompareOp::Lt,
        field: PayloadKey::Amount.to_string(),
    };

    // Create a new RuleSet.
    let mut second_rule_set = RuleSetV1::new("test rule_set".to_string(), context.payer.pubkey());
    second_rule_set
        .add(
            Operation::Transfer {
                scenario: utils::TransferScenario::Holder,
            }
            .to_string(),
            amount_check,
        )
        .unwrap();

    // Put the updated RuleSet on chain.
    let rule_set_addr = create_rule_set_on_chain!(
        &mut context,
        second_rule_set.clone(),
        "test rule_set".to_string()
    )
    .await;

    // --------------------------------
    // Validate fail when trying to index a RuleSet revision that does not exist.
    // --------------------------------
    // Create a Keypair to simulate a token mint address.
    let mint = Keypair::new().pubkey();

    // Store a payload of data with an amount allowed by the the second revision `RuleSet`.
    let payload = Payload::from([(PayloadKey::Amount.to_string(), PayloadType::Number(2))]);

    // Create a `validate` instruction with the additional signer pubkey added as a signer, but with
    // an unknown RuleSet revision.
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
            rule_set_revision: Some(3),
        })
        .unwrap()
        .instruction();

    // Fail to validate Transfer operation.
    let err =
        process_failing_validate_ix!(&mut context, validate_ix, vec![&additional_signer], None)
            .await;

    // Check that error is what we expect.
    assert_custom_error!(err, RuleSetError::RuleSetRevisionNotAvailable);
}

#[tokio::test]
async fn test_correct_rule_set_is_used_after_update() {
    let mut context = program_test().start_with_context().await;

    // --------------------------------
    // Create RuleSets
    // --------------------------------
    let additional_signer = Keypair::new();
    let adtl_signer_rule = Rule::AdditionalSigner {
        account: additional_signer.pubkey(),
    };

    // Create a RuleSet.
    let mut first_rule_set = RuleSetV1::new("test rule_set".to_string(), context.payer.pubkey());
    first_rule_set
        .add(
            Operation::Transfer {
                scenario: utils::TransferScenario::Holder,
            }
            .to_string(),
            adtl_signer_rule,
        )
        .unwrap();

    // Put the RuleSet on chain.
    let _rule_set_addr = create_rule_set_on_chain!(
        &mut context,
        first_rule_set.clone(),
        "test rule_set".to_string()
    )
    .await;

    let amount_check = Rule::Amount {
        amount: 10,
        operator: CompareOp::Lt,
        field: PayloadKey::Amount.to_string(),
    };

    // Create a new RuleSet.
    let mut second_rule_set = RuleSetV1::new("test rule_set".to_string(), context.payer.pubkey());
    second_rule_set
        .add(
            Operation::Transfer {
                scenario: utils::TransferScenario::Holder,
            }
            .to_string(),
            amount_check,
        )
        .unwrap();

    // Put the updated RuleSet on chain.
    let rule_set_addr = create_rule_set_on_chain!(
        &mut context,
        second_rule_set.clone(),
        "test rule_set".to_string()
    )
    .await;

    // --------------------------------
    // Validate that when using first RuleSet, we fail based on inputs that would pass second RuleSet.
    // --------------------------------
    // Create a Keypair to simulate a token mint address.
    let mint = Keypair::new().pubkey();

    // Store a payload of data with an amount allowed by the the second revision `RuleSet` but ignored
    // by the first `RuleSet`.
    let payload = Payload::from([(PayloadKey::Amount.to_string(), PayloadType::Number(2))]);

    // Create a `validate` instruction with the additional signer pubkey added but not sent as a signer.
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
            rule_set_revision: Some(0),
        })
        .unwrap()
        .instruction();

    // Fail to validate Transfer operation.
    let err = process_failing_validate_ix!(&mut context, validate_ix, vec![], None).await;

    // Check that error is what we expect.
    assert_custom_error!(err, RuleSetError::AdditionalSignerCheckFailed);

    // --------------------------------
    // Validate that when using first RuleSet, we pass based on inputs that would fail second RuleSet.
    // --------------------------------
    // Store a payload of data with an amount not allowed by the the second revision `RuleSet` but ignored
    // by the first `RuleSet`.
    let payload = Payload::from([(PayloadKey::Amount.to_string(), PayloadType::Number(200))]);

    // Create a `validate` instruction with the additional signer pubkey added as a signer.
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
            rule_set_revision: Some(0),
        })
        .unwrap()
        .instruction();

    // Validate Transfer operation.
    process_passing_validate_ix!(&mut context, validate_ix, vec![&additional_signer], None).await;

    // --------------------------------
    // Validate that when using second RuleSet, we fail based on inputs that would pass first RuleSet.
    // --------------------------------
    // Store a payload of data with an amount not allowed by the the second revision `RuleSet` but ignored
    // by the first `RuleSet`.
    let payload = Payload::from([(PayloadKey::Amount.to_string(), PayloadType::Number(200))]);

    // Create a `validate` instruction with the additional signer pubkey added as a signer.
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
            rule_set_revision: Some(1),
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
    // Validate that when using second RuleSet, we pass based on inputs that would fail first RuleSet.
    // --------------------------------
    // Store a payload of data with an amount allowed by the the second revision `RuleSet` but ignored
    // by the first `RuleSet`.
    let payload = Payload::from([(PayloadKey::Amount.to_string(), PayloadType::Number(2))]);

    // Create a `validate` instruction with the additional signer pubkey added but not sent as a signer.
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
            rule_set_revision: Some(1),
        })
        .unwrap()
        .instruction();

    // Validate Transfer operation.
    process_passing_validate_ix!(&mut context, validate_ix, vec![], None).await;
}
