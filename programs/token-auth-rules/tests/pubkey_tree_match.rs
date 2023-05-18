#![cfg(feature = "test-bpf")]

pub mod utils;

use mpl_token_auth_rules::{
    error::RuleSetError,
    instruction::{builders::ValidateBuilder, InstructionBuilder, ValidateArgs},
    payload::{Payload, PayloadType, ProofInfo},
    state::{Rule, RuleSetV1},
};
use solana_program::pubkey::Pubkey;
use solana_program_test::tokio;
use solana_sdk::{signature::Signer, signer::keypair::Keypair};
use utils::{program_test, Operation, PayloadKey};

#[tokio::test]
async fn pubkey_tree_match() {
    let mut context = program_test().start_with_context().await;

    // --------------------------------
    // Create RuleSet
    // --------------------------------
    // Merkle tree root generated in a different test program.
    let tree_root: [u8; 32] = [
        132, 141, 27, 31, 23, 154, 145, 128, 32, 62, 122, 224, 248, 128, 37, 139, 200, 46, 163,
        238, 76, 123, 155, 141, 73, 12, 111, 192, 122, 80, 126, 155,
    ];

    // Create a Rule: The provided leaf node must be a
    // member of the marketplace Merkle tree.
    let rule = Rule::PubkeyTreeMatch {
        root: tree_root,
        pubkey_field: PayloadKey::Authority.to_string(),
        proof_field: PayloadKey::AuthorityProof.to_string(),
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

    // Merkle tree leaf node generated in a different test program.
    let leaf: [u8; 32] = [
        2, 157, 245, 156, 21, 37, 147, 96, 42, 190, 206, 14, 24, 1, 106, 49, 167, 236, 38, 73, 98,
        53, 60, 9, 154, 31, 240, 126, 210, 197, 76, 7,
    ];

    // Convert it to a Pubkey.
    let leaf = Pubkey::from(leaf);

    // INCORRECT Merkle tree proof generated in a different test program.  One value is corrupted.
    let incorrect_proof: Vec<[u8; 32]> = vec![
        [
            246, 54, 96, 185, 234, 119, 124, 220, 54, 137, 25, 200, 18, 12, 114, 75, 211, 203, 154,
            229, 197, 53, 164, 84, 38, 56, 20, 74, 192, 119, 37, 175,
        ],
        [
            193, 84, 42, 232, 119, 107, 227, 166, 30, 233, 40, 10, 51, 229, 90, 59, 165, 212, 67,
            193, 159, 126, 26, 200, 13, 209, 162, 98, 52, 125, 240, 77,
        ],
        [
            238, 14, 13, 214, 124, 172, 89, 7, 66, 168, 226, 88, 92, 22, 18, 17, 94, 96, 37, 234,
            101, 96, 129, 26, 137, 222, 96, 86, 245, 11, 199, 140,
        ],
    ];

    let incorrect_proof_info = ProofInfo::new(incorrect_proof);

    // Store the payload of data to validate against the rule definition, with an INCORRECT proof.
    let payload = Payload::from([
        (PayloadKey::Authority.to_string(), PayloadType::Pubkey(leaf)),
        (
            PayloadKey::AuthorityProof.to_string(),
            PayloadType::MerkleProof(incorrect_proof_info),
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
    let err = process_failing_validate_ix!(&mut context, validate_ix, vec![], None).await;

    // Check that error is what we expect.
    assert_custom_error!(err, RuleSetError::PubkeyTreeMatchCheckFailed);

    // --------------------------------
    // Validate pass
    // --------------------------------
    // CORRECT Merkle tree proof generated in a different test program.
    let correct_proof: Vec<[u8; 32]> = vec![
        [
            246, 54, 96, 185, 234, 119, 124, 220, 54, 137, 25, 200, 18, 12, 114, 75, 211, 203, 154,
            229, 197, 53, 164, 84, 38, 56, 20, 74, 192, 119, 37, 175,
        ],
        [
            193, 84, 33, 232, 119, 107, 227, 166, 30, 233, 40, 10, 51, 229, 90, 59, 165, 212, 67,
            193, 159, 126, 26, 200, 13, 209, 162, 98, 52, 125, 240, 77,
        ],
        [
            238, 14, 13, 214, 124, 172, 89, 7, 66, 168, 226, 88, 92, 22, 18, 17, 94, 96, 37, 234,
            101, 96, 129, 26, 137, 222, 96, 86, 245, 11, 199, 140,
        ],
    ];

    let correct_proof_info = ProofInfo::new(correct_proof);

    // Store the payload of data to validate against the rule definition, with a CORRECT proof.
    let payload = Payload::from([
        (PayloadKey::Authority.to_string(), PayloadType::Pubkey(leaf)),
        (
            PayloadKey::AuthorityProof.to_string(),
            PayloadType::MerkleProof(correct_proof_info),
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
