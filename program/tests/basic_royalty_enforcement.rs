#![cfg(feature = "test-bpf")]

pub mod utils;

use rmp_serde::Serializer;
use serde::Serialize;
use solana_program_test::tokio;
use solana_sdk::{signature::Signer, transaction::Transaction};
use token_authorization_rules::state::{Operation, Rule, RuleSet};
use utils::program_test;

#[tokio::test]
async fn basic_royalty_enforcement() {
    let mut context = program_test().start_with_context().await;

    // Find RuleSet PDA.
    let (ruleset_addr, _ruleset_bump) = token_authorization_rules::pda::find_ruleset_address(
        context.payer.pubkey(),
        "basic_royalty_enforcement".to_string(),
    );

    // Rule for Transfers.  Initially the rule was to allow transfers to
    // either a Token Owned Escrow OR an Identity-associated wallet.  But
    // Identity was descoped.
    let program_is_token_metadata = Rule::ProgramOwned {
        program: mpl_token_metadata::id(),
    };

    let marketplace_tree_root = [0u8; 32];

    // Rule for Delegate and SaleTransfer.  The provided leaf node must be a
    // member of the marketplace Merkle tree.
    let leaf_in_marketplace_tree = Rule::PubkeyTreeMatch {
        root: marketplace_tree_root,
    };

    // Create Basic Royalty Enforcement Ruleset.
    let mut basic_royalty_enforcement_rule_set = RuleSet::new();
    basic_royalty_enforcement_rule_set.add(Operation::Transfer, program_is_token_metadata);
    basic_royalty_enforcement_rule_set.add(Operation::Delegate, leaf_in_marketplace_tree.clone());
    basic_royalty_enforcement_rule_set.add(Operation::SaleTransfer, leaf_in_marketplace_tree);

    println!(
        "{}",
        serde_json::to_string_pretty(&basic_royalty_enforcement_rule_set,).unwrap()
    );

    // Serialize the RuleSet using RMP serde.
    let mut serialized_data = Vec::new();
    basic_royalty_enforcement_rule_set
        .serialize(&mut Serializer::new(&mut serialized_data))
        .unwrap();

    // Create a `create` instruction.
    let create_ix = token_authorization_rules::instruction::create(
        token_authorization_rules::id(),
        context.payer.pubkey(),
        ruleset_addr,
        "basic_royalty_enforcement".to_string(),
        serialized_data,
    );

    // Add it to a transaction.
    let create_tx = Transaction::new_signed_with_payer(
        &[create_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    // Process the transaction.
    context
        .banks_client
        .process_transaction(create_tx)
        .await
        .expect("creation should succeed");
}
