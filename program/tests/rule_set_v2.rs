#![cfg(feature = "test-bpf")]

pub mod utils;

use borsh::BorshDeserialize;
use mpl_token_auth_rules::{
    state::{
        CompareOp, Rule, RuleSetHeader, RuleSetRevisionMapV1, RuleSetV1,
        RULE_SET_SERIALIZED_HEADER_LEN,
    },
    state_v2::{All, Amount, ProgramOwnedList, RuleSetV2},
    LibVersion,
};
use solana_program::{pubkey, pubkey::Pubkey};
use solana_program_test::{tokio, ProgramTestContext};
use solana_sdk::{commitment_config::CommitmentLevel, signature::Signer, signer::keypair::Keypair};
use utils::{
    program_test, DelegateScenario, MetadataDelegateRole, Operation, PayloadKey, TokenDelegateRole,
    TransferScenario,
};

const ADDITIONAL_COMPUTE: u32 = 400_000;
const RULE_SET_NAME: &str = "Metaplex Royalty RuleSet Dev";

// --------------------------------
// Define Program Allow List
// --------------------------------
const ROOSTER_PROGRAM_ID: Pubkey = pubkey!("Roostrnex2Z9Y2XZC49sFAdZARP8E4iFpEnZC5QJWdz");
const TOKEN_METADATA_PROGRAM_ID: Pubkey = pubkey!("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s");
const TOKEN_AUTH_RULES_ID: Pubkey = pubkey!("auth9SigNpDKz4sJJ1DfCTuZrZNSAgh9sFD3rboVmgg");

const TRANSFER_PROGRAM_BASE_ALLOW_LIST: [Pubkey; 3] = [
    TOKEN_METADATA_PROGRAM_ID,
    ROOSTER_PROGRAM_ID,
    TOKEN_AUTH_RULES_ID,
];
const DELEGATE_PROGRAM_BASE_ALLOW_LIST: [Pubkey; 3] = [
    TOKEN_METADATA_PROGRAM_ID,
    ROOSTER_PROGRAM_ID,
    TOKEN_AUTH_RULES_ID,
];
const ADVANCED_DELEGATE_PROGRAM_BASE_ALLOW_LIST: [Pubkey; 3] = [
    TOKEN_METADATA_PROGRAM_ID,
    ROOSTER_PROGRAM_ID,
    TOKEN_AUTH_RULES_ID,
];

struct ComposedRulesV1 {
    transfer_rule: Rule,
    delegate_rule: Rule,
    advanced_delegate_rule: Rule,
}

struct ComposedRulesV2 {
    transfer_rule: Vec<u8>,
    delegate_rule: Vec<u8>,
    advanced_delegate_rule: Vec<u8>,
}

// Get the four Composed Rules used in this RuleSet.
fn get_composed_rules_v2() -> ComposedRulesV2 {
    // --------------------------------
    // Create Primitive Rules
    // --------------------------------

    let nft_amount = Amount::serialize(
        1,
        mpl_token_auth_rules::state_v2::CompareOp::Eq,
        PayloadKey::Amount.to_string(),
    )
    .unwrap();

    // Generate some random programs to add to the base lists.
    let random_programs = (0..200)
        .map(|_| Keypair::new().pubkey())
        .collect::<Vec<_>>();

    let multi_field_program_allow_list = ProgramOwnedList::serialize(
        format!(
            "{}|{}|{}",
            PayloadKey::Source.to_string(),
            PayloadKey::Destination.to_string(),
            PayloadKey::Authority.to_string()
        ),
        &[
            TRANSFER_PROGRAM_BASE_ALLOW_LIST.to_vec(),
            random_programs.clone(),
        ]
        .concat(),
    )
    .unwrap();

    let delegate_program_allow_list = ProgramOwnedList::serialize(
        PayloadKey::Delegate.to_string(),
        &[
            DELEGATE_PROGRAM_BASE_ALLOW_LIST.to_vec(),
            random_programs.clone(),
        ]
        .concat(),
    )
    .unwrap();

    let advanced_delegate_program_allow_list = ProgramOwnedList::serialize(
        PayloadKey::Delegate.to_string(),
        &[
            ADVANCED_DELEGATE_PROGRAM_BASE_ALLOW_LIST.to_vec(),
            random_programs,
        ]
        .concat(),
    )
    .unwrap();

    // --------------------------------
    // Create Composed Rules from
    // Primitive Rules
    // --------------------------------
    // amount is 1 && (source owner on allow list || dest owner on allow list || authority owner on allow list )
    let transfer_rule = All::serialize(&[&nft_amount, &multi_field_program_allow_list]).unwrap();

    let delegate_rule = All::serialize(&[&nft_amount, &delegate_program_allow_list]).unwrap();

    let advanced_delegate_rule =
        All::serialize(&[&nft_amount, &advanced_delegate_program_allow_list]).unwrap();

    ComposedRulesV2 {
        transfer_rule,
        delegate_rule,
        advanced_delegate_rule,
    }
}

fn get_royalty_rule_set_v2(owner: Pubkey) -> Vec<u8> {
    // Get transfer and wallet-to-wallet rules.
    let composed_rules = get_composed_rules_v2();

    let mut operations = Vec::new();
    let mut rules = Vec::new();

    // --------------------------------
    // Set up transfer operations
    // --------------------------------

    let transfer_transfer_delegate_operation = Operation::Transfer {
        scenario: TransferScenario::TransferDelegate,
    };

    operations.push(transfer_transfer_delegate_operation.to_string());
    rules.push(composed_rules.transfer_rule);

    // --------------------------------
    // Setup metadata delegate operations
    // --------------------------------

    let metadata_delegate_authority_operation = Operation::Delegate {
        scenario: DelegateScenario::Metadata(MetadataDelegateRole::Authority),
    };

    operations.push(metadata_delegate_authority_operation.to_string());
    rules.push(composed_rules.delegate_rule);

    // --------------------------------
    // Setup token delegate operations
    // --------------------------------

    let token_delegate_locked_transfer_operation = Operation::Delegate {
        scenario: DelegateScenario::Token(TokenDelegateRole::LockedTransfer),
    };

    operations.push(token_delegate_locked_transfer_operation.to_string());
    rules.push(composed_rules.advanced_delegate_rule);

    RuleSetV2::serialize(owner, RULE_SET_NAME, operations.as_slice(), &rules).unwrap()
}

async fn create_royalty_rule_set_v2(context: &mut ProgramTestContext) -> Pubkey {
    let royalty_rule_set = get_royalty_rule_set_v2(context.payer.pubkey());

    let clone = royalty_rule_set.clone();
    let rule_set = RuleSetV2::from_bytes(&clone).unwrap();
    println!("{}", rule_set);

    // Put the `RuleSet` on chain.
    create_big_rule_set_v2_on_chain!(
        context,
        royalty_rule_set,
        RULE_SET_NAME.to_string(),
        Some(ADDITIONAL_COMPUTE)
    )
    .await
}

// Get the four Composed Rules used in this RuleSet.
fn get_composed_rules_v1() -> ComposedRulesV1 {
    // --------------------------------
    // Create Primitive Rules
    // --------------------------------
    let nft_amount = Rule::Amount {
        field: PayloadKey::Amount.to_string(),
        amount: 1,
        operator: CompareOp::Eq,
    };

    // Generate some random programs to add to the base lists.
    let random_programs = (0..30).map(|_| Keypair::new().pubkey()).collect::<Vec<_>>();

    let multi_field_program_allow_list = Rule::ProgramOwnedList {
        programs: [
            TRANSFER_PROGRAM_BASE_ALLOW_LIST.to_vec(),
            random_programs.clone(),
        ]
        .concat(),
        field: format!(
            "{}|{}|{}",
            PayloadKey::Source.to_string(),
            PayloadKey::Destination.to_string(),
            PayloadKey::Authority.to_string()
        ),
    };

    let delegate_program_allow_list = Rule::ProgramOwnedList {
        programs: [
            DELEGATE_PROGRAM_BASE_ALLOW_LIST.to_vec(),
            random_programs.clone(),
        ]
        .concat(),
        field: PayloadKey::Delegate.to_string(),
    };

    let advanced_delegate_program_allow_list = Rule::ProgramOwnedList {
        programs: [
            ADVANCED_DELEGATE_PROGRAM_BASE_ALLOW_LIST.to_vec(),
            random_programs,
        ]
        .concat(),
        field: PayloadKey::Delegate.to_string(),
    };

    // --------------------------------
    // Create Composed Rules from
    // Primitive Rules
    // --------------------------------
    // amount is 1 && (source owner on allow list || dest owner on allow list || authority owner on allow list )
    let transfer_rule = Rule::All {
        rules: vec![nft_amount.clone(), multi_field_program_allow_list],
    };

    let delegate_rule = Rule::All {
        rules: vec![nft_amount.clone(), delegate_program_allow_list],
    };

    let advanced_delegate_rule = Rule::All {
        rules: vec![nft_amount, advanced_delegate_program_allow_list],
    };

    ComposedRulesV1 {
        transfer_rule,
        delegate_rule,
        advanced_delegate_rule,
    }
}

fn get_royalty_rule_set_v1(owner: Pubkey) -> RuleSetV1 {
    // Create a RuleSet.
    let mut royalty_rule_set = RuleSetV1::new(RULE_SET_NAME.to_string(), owner);

    // Get transfer and wallet-to-wallet rules.
    let rules = get_composed_rules_v1();

    // --------------------------------
    // Set up transfer operations
    // --------------------------------

    let transfer_transfer_delegate_operation = Operation::Transfer {
        scenario: TransferScenario::TransferDelegate,
    };

    royalty_rule_set
        .add(
            transfer_transfer_delegate_operation.to_string(),
            rules.transfer_rule.clone(),
        )
        .unwrap();

    // --------------------------------
    // Setup metadata delegate operations
    // --------------------------------

    let metadata_delegate_authority_operation = Operation::Delegate {
        scenario: DelegateScenario::Metadata(MetadataDelegateRole::Authority),
    };

    royalty_rule_set
        .add(
            metadata_delegate_authority_operation.to_string(),
            rules.delegate_rule.clone(),
        )
        .unwrap();

    // --------------------------------
    // Setup token delegate operations
    // --------------------------------

    let token_delegate_locked_transfer_operation = Operation::Delegate {
        scenario: DelegateScenario::Token(TokenDelegateRole::LockedTransfer),
    };

    // --------------------------------
    // NOTE THIS IS THE ONLY OPERATION
    // THAT USES THE ADVANCED DELEGATE
    // RULE.
    // --------------------------------
    royalty_rule_set
        .add(
            token_delegate_locked_transfer_operation.to_string(),
            rules.advanced_delegate_rule,
        )
        .unwrap();

    royalty_rule_set
}

async fn create_royalty_rule_set_v1(context: &mut ProgramTestContext) -> Pubkey {
    let royalty_rule_set = get_royalty_rule_set_v1(context.payer.pubkey());

    print!("Royalty Rule Set: {:#?}", royalty_rule_set);

    // Put the `RuleSet` on chain.
    create_big_rule_set_on_chain!(
        context,
        royalty_rule_set.clone(),
        RULE_SET_NAME.to_string(),
        Some(ADDITIONAL_COMPUTE)
    )
    .await
}

// ------------------------------------------------------------------------- //
// Tests                                                                     //
// ------------------------------------------------------------------------- //

#[tokio::test]
async fn create_rule_set_v1() {
    let mut context = program_test().start_with_context().await;
    let _rule_set_addr = create_royalty_rule_set_v1(&mut context).await;
}

#[tokio::test]
async fn create_rule_set_v2() {
    let mut context = program_test().start_with_context().await;
    let _rule_set_addr = create_royalty_rule_set_v2(&mut context).await;
}

#[tokio::test]
async fn create_rule_set_with_v1_and_v2() {
    let mut context = program_test().start_with_context().await;

    // --------------------------------
    // Create RuleSetV1 revision 0
    // --------------------------------

    let royalty_rule_set = get_royalty_rule_set_v1(context.payer.pubkey());

    let rule_set_addr = create_big_rule_set_on_chain!(
        &mut context,
        royalty_rule_set,
        RULE_SET_NAME.to_string(),
        Some(ADDITIONAL_COMPUTE)
    )
    .await;

    let rule_set_account = context
        .banks_client
        .get_account_with_commitment(rule_set_addr, CommitmentLevel::Processed)
        .await
        .expect("account not found")
        .expect("account empty");

    let account_length_revision_0 = rule_set_account.data.len();

    // --------------------------------
    // Create RuleSetV2 revision 1 and update on chain
    // --------------------------------

    let royalty_rule_set = get_royalty_rule_set_v2(context.payer.pubkey());

    // Put the `RuleSet` on chain.
    let _rule_set_addr = create_big_rule_set_v2_on_chain!(
        &mut context,
        royalty_rule_set,
        RULE_SET_NAME.to_string(),
        Some(ADDITIONAL_COMPUTE)
    )
    .await;

    let rule_set_account = context
        .banks_client
        .get_account_with_commitment(rule_set_addr, CommitmentLevel::Processed)
        .await
        .expect("account not found")
        .expect("account empty");

    // make sure the revision was stored on chain
    assert!(rule_set_account.data.len() > account_length_revision_0);

    // --------------------------------
    // Validate the on chain data for all RuleSets
    // --------------------------------

    let data = rule_set_account.data;

    let header = RuleSetHeader::try_from_slice(&data[..RULE_SET_SERIALIZED_HEADER_LEN])
        .expect("Failed to deserialize RuleSetHeader");

    let location = header.rev_map_version_location;
    // the revision map is stored at location + 1, since the first byte is the version
    let revision_map = RuleSetRevisionMapV1::try_from_slice(&data[location + 1..])
        .expect("Failed to deserialize RuleSetRevisionMapV1");

    let rule_set_v1 = rmp_serde::from_slice::<RuleSetV1>(
        &data[revision_map.rule_set_revisions[0] + 1..revision_map.rule_set_revisions[1]],
    )
    .expect("Failed to deserialize RuleSetV1");

    assert_eq!(rule_set_v1.lib_version(), LibVersion::V1 as u8);
    assert_eq!(rule_set_v1.operations.len(), 3);

    let rule_set_v2 = RuleSetV2::from_bytes(&data[revision_map.rule_set_revisions[1]..location])
        .expect("Failed to deserialize RuleSetV2");

    assert_eq!(rule_set_v2.lib_version(), LibVersion::V2 as u8);
    assert_eq!(rule_set_v2.operations.len(), 3);
}
