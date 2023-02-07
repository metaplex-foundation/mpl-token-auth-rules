#![cfg(feature = "test-bpf")]

pub mod utils;

use mpl_token_auth_rules::{
    error::RuleSetError,
    instruction::{builders::WriteToBufferBuilder, InstructionBuilder, WriteToBufferArgs},
    state::{Rule, RuleSetV1},
};
use rmp_serde::Serializer;
use serde::Serialize;
use solana_program_test::tokio;
use solana_sdk::{signature::Signer, signer::keypair::Keypair, transaction::Transaction};
use utils::{program_test, Operation};

#[tokio::test]
#[should_panic]
async fn write_to_buffer_payer_not_signer_panics() {
    let mut context = program_test().start_with_context().await;

    // --------------------------------
    // Create RuleSet
    // --------------------------------
    // Create some rules.
    let adtl_signer = Rule::AdditionalSigner {
        account: context.payer.pubkey(),
    };

    // Create a RuleSet.
    let mut rule_set = RuleSetV1::new("test rule_set".to_string(), context.payer.pubkey());
    rule_set
        .add(
            Operation::Transfer {
                scenario: utils::TransferScenario::Holder,
            }
            .to_string(),
            adtl_signer,
        )
        .unwrap();

    // Serialize the RuleSet using RMP serde.
    let mut serialized_rule_set = Vec::new();
    rule_set
        .serialize(&mut Serializer::new(&mut serialized_rule_set))
        .unwrap();

    // Get one partial chunk of the serialized `RuleSet`.
    let serialized_rule_set_chunk = serialized_rule_set.chunks(100).next().unwrap();

    let (buffer_pda, _buffer_bump) =
        mpl_token_auth_rules::pda::find_buffer_address(context.payer.pubkey());

    // Create a `write_to_buffer` instruction.
    let other_payer = Keypair::new();
    let write_to_buffer_ix = WriteToBufferBuilder::new()
        .payer(other_payer.pubkey())
        .buffer_pda(buffer_pda)
        .build(WriteToBufferArgs::V1 {
            serialized_rule_set: serialized_rule_set_chunk.to_vec(),
            overwrite: true,
        })
        .unwrap()
        .instruction();

    // Add it to a transaction.
    let write_to_buffer_tx = Transaction::new_signed_with_payer(
        &[write_to_buffer_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    // Process the transaction.  It will panic because of not enough signers.
    let _result = context
        .banks_client
        .process_transaction(write_to_buffer_tx)
        .await;
}

#[tokio::test]
async fn write_to_buffer_wrong_pda_fails() {
    let mut context = program_test().start_with_context().await;

    // --------------------------------
    // Create RuleSet
    // --------------------------------
    // Create some rules.
    let adtl_signer = Rule::AdditionalSigner {
        account: context.payer.pubkey(),
    };

    // Create a RuleSet.
    let mut rule_set = RuleSetV1::new("test rule_set".to_string(), context.payer.pubkey());
    rule_set
        .add(
            Operation::Transfer {
                scenario: utils::TransferScenario::Holder,
            }
            .to_string(),
            adtl_signer,
        )
        .unwrap();

    // Serialize the RuleSet using RMP serde.
    let mut serialized_rule_set = Vec::new();
    rule_set
        .serialize(&mut Serializer::new(&mut serialized_rule_set))
        .unwrap();

    // Get one partial chunk of the serialized `RuleSet`.
    let serialized_rule_set_chunk = serialized_rule_set.chunks(100).next().unwrap();

    // Find buffer PDA using WRONG creator for seed.
    let wrong_creator = Keypair::new();
    let (buffer_pda, _buffer_bump) =
        mpl_token_auth_rules::pda::find_buffer_address(wrong_creator.pubkey());

    // Create a `write_to_buffer` instruction.
    let write_to_buffer_ix = WriteToBufferBuilder::new()
        .payer(context.payer.pubkey())
        .buffer_pda(buffer_pda)
        .build(WriteToBufferArgs::V1 {
            serialized_rule_set: serialized_rule_set_chunk.to_vec(),
            overwrite: true,
        })
        .unwrap()
        .instruction();

    // Add it to a transaction.
    let write_to_buffer_tx = Transaction::new_signed_with_payer(
        &[write_to_buffer_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    // Process the transaction.
    let err = context
        .banks_client
        .process_transaction(write_to_buffer_tx)
        .await
        .expect_err("Write buffer should fail");

    // Check that error is what we expect.
    assert_custom_error!(err, RuleSetError::DerivedKeyInvalid);
}
