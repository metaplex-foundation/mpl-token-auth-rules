#![cfg(feature = "test-bpf")]

pub mod utils;

use mpl_token_auth_rules::{
    error::RuleSetError,
    instruction::{
        builders::{CreateOrUpdateBuilder, WriteToBufferBuilder},
        CreateOrUpdateArgs, InstructionBuilder, WriteToBufferArgs,
    },
    state::{Rule, RuleSetV1},
};
use rmp_serde::Serializer;
use serde::Serialize;
use solana_program_test::tokio;
use solana_sdk::{signature::Signer, signer::keypair::Keypair, transaction::Transaction};
use utils::{program_test, Operation};

#[tokio::test]
#[should_panic]
async fn create_payer_not_signer_panics() {
    let mut context = program_test().start_with_context().await;

    // --------------------------------
    // Create RuleSet
    // --------------------------------
    // Create a Pass Rule.
    let pass_rule = Rule::Pass;

    // Create a RuleSet.
    let other_payer = Keypair::new();
    let mut rule_set = RuleSetV1::new("test rule_set".to_string(), other_payer.pubkey());
    rule_set
        .add(
            Operation::Transfer {
                scenario: utils::TransferScenario::Holder,
            }
            .to_string(),
            pass_rule,
        )
        .unwrap();

    // Find RuleSet PDA.
    let (rule_set_addr, _rule_set_bump) = mpl_token_auth_rules::pda::find_rule_set_address(
        other_payer.pubkey(),
        "test rule_set".to_string(),
    );

    // Serialize the RuleSet using RMP serde.
    let mut serialized_rule_set = Vec::new();
    rule_set
        .serialize(&mut Serializer::new(&mut serialized_rule_set))
        .unwrap();

    // Create a `create` instruction with a payer that won't be a signer.
    let create_ix = CreateOrUpdateBuilder::new()
        .payer(other_payer.pubkey())
        .rule_set_pda(rule_set_addr)
        .build(CreateOrUpdateArgs::V1 {
            serialized_rule_set,
        })
        .unwrap()
        .instruction();

    // Add it to a transaction but don't add other payer as a signer.
    let create_tx = Transaction::new_signed_with_payer(
        &[create_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    // Process the transaction.  It will panic because of not enough signers.
    let _result = context.banks_client.process_transaction(create_tx).await;
}

#[tokio::test]
async fn create_rule_set_empty_buffer_fails() {
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

    // --------------------------------
    // Fail on-chain creation
    // --------------------------------
    // Find RuleSet PDA.
    let (rule_set_addr, _rule_set_bump) = mpl_token_auth_rules::pda::find_rule_set_address(
        context.payer.pubkey(),
        "test rule_set".to_string(),
    );

    let (buffer_pda, _buffer_bump) =
        mpl_token_auth_rules::pda::find_buffer_address(context.payer.pubkey());

    // Create a `create` instruction.  We are adding an uninitialized buffer as an extra account,
    // which will be used instead of the `serialized_rule_set` arg that is passed in.  Normally
    // when passing in a buffer account, the `serialized_rule_set` would just be an empty Vec, but
    // for this test we are trying to prove that we fail due to the empty buffer, so we pass in a
    // `serialized_rule_set` that would otherwise pass.
    let create_ix = CreateOrUpdateBuilder::new()
        .payer(context.payer.pubkey())
        .rule_set_pda(rule_set_addr)
        .buffer_pda(buffer_pda)
        .build(CreateOrUpdateArgs::V1 {
            serialized_rule_set,
        })
        .unwrap()
        .instruction();

    // Add it to a transaction.
    let create_tx = Transaction::new_signed_with_payer(
        &[create_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    // Process the transaction.
    let err = context
        .banks_client
        .process_transaction(create_tx)
        .await
        .expect_err("Creation should fail");

    // Check that error is what we expect.
    assert_custom_error!(err, RuleSetError::MessagePackDeserializationError);
}

#[tokio::test]
async fn create_rule_set_partial_buffer_fails() {
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
    context
        .banks_client
        .process_transaction(write_to_buffer_tx)
        .await
        .unwrap();

    // --------------------------------
    // Fail on-chain creation
    // --------------------------------
    // Find RuleSet PDA.
    let (rule_set_addr, _rule_set_bump) = mpl_token_auth_rules::pda::find_rule_set_address(
        context.payer.pubkey(),
        "test rule_set".to_string(),
    );

    // Create a `create` instruction.  We are adding an partially written buffer as an extra account,
    // which will be used instead of the `serialized_rule_set` arg that is passed in.  Normally
    // when passing in a buffer account, the `serialized_rule_set` would just be an empty Vec, but
    // for this test we are trying to prove that we fail due to the partial buffer, so we pass in a
    // `serialized_rule_set` that would otherwise pass.
    let create_ix = CreateOrUpdateBuilder::new()
        .payer(context.payer.pubkey())
        .rule_set_pda(rule_set_addr)
        .buffer_pda(buffer_pda)
        .build(CreateOrUpdateArgs::V1 {
            serialized_rule_set,
        })
        .unwrap()
        .instruction();

    // Add it to a transaction.
    let create_tx = Transaction::new_signed_with_payer(
        &[create_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    // Process the transaction.
    let err = context
        .banks_client
        .process_transaction(create_tx)
        .await
        .expect_err("Creation should fail");

    // Check that error is what we expect.
    assert_custom_error!(err, RuleSetError::MessagePackDeserializationError);
}

#[tokio::test]
async fn create_rule_set_empty_rule_set_fails() {
    let mut context = program_test().start_with_context().await;

    // --------------------------------
    // Fail on-chain creation
    // --------------------------------
    // Find RuleSet PDA.
    let (rule_set_addr, _rule_set_bump) = mpl_token_auth_rules::pda::find_rule_set_address(
        context.payer.pubkey(),
        "test rule_set".to_string(),
    );

    // Create a `create` instruction with an empty Vec for a `RuleSet`.
    let create_ix = CreateOrUpdateBuilder::new()
        .payer(context.payer.pubkey())
        .rule_set_pda(rule_set_addr)
        .build(CreateOrUpdateArgs::V1 {
            serialized_rule_set: vec![],
        })
        .unwrap()
        .instruction();

    // Add it to a transaction.
    let create_tx = Transaction::new_signed_with_payer(
        &[create_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    // Process the transaction.
    let err = context
        .banks_client
        .process_transaction(create_tx)
        .await
        .expect_err("Creation should fail");

    // Check that error is what we expect.
    assert_custom_error!(err, RuleSetError::MessagePackDeserializationError);
}

#[tokio::test]
async fn create_rule_set_name_too_long_fails() {
    let mut context = program_test().start_with_context().await;

    // --------------------------------
    // Create RuleSet
    // --------------------------------
    // Create some rules.
    let adtl_signer = Rule::AdditionalSigner {
        account: context.payer.pubkey(),
    };

    // Create a RuleSet.
    let mut rule_set = RuleSetV1::new(
        "test rule_set that has too long of a name".to_string(),
        context.payer.pubkey(),
    );

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

    // --------------------------------
    // Fail on-chain creation
    // --------------------------------
    // Find RuleSet PDA.  This isn't the correct PDA but we expect to fail because the name in the
    // serialized `RuleSet` is too long.
    let (rule_set_addr, _rule_set_bump) = mpl_token_auth_rules::pda::find_rule_set_address(
        context.payer.pubkey(),
        "test rule_set".to_string(),
    );

    // Create a `create` instruction.
    let create_ix = CreateOrUpdateBuilder::new()
        .payer(context.payer.pubkey())
        .rule_set_pda(rule_set_addr)
        .build(CreateOrUpdateArgs::V1 {
            serialized_rule_set,
        })
        .unwrap()
        .instruction();

    // Add it to a transaction.
    let create_tx = Transaction::new_signed_with_payer(
        &[create_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    // Process the transaction.
    let err = context
        .banks_client
        .process_transaction(create_tx)
        .await
        .expect_err("Creation should fail");

    // Check that error is what we expect.
    assert_custom_error!(err, RuleSetError::NameTooLong);
}

#[tokio::test]
async fn create_rule_set_wrong_owner_fails() {
    let mut context = program_test().start_with_context().await;

    // --------------------------------
    // Create RuleSet
    // --------------------------------
    // Create some rules.
    let adtl_signer = Rule::AdditionalSigner {
        account: context.payer.pubkey(),
    };

    // Create a RuleSet.
    let mut rule_set = RuleSetV1::new("test rule_set".to_string(), Keypair::new().pubkey());
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

    // --------------------------------
    // Fail on-chain creation
    // --------------------------------
    // Find RuleSet PDA with DIFFERENT NAME.
    let (rule_set_addr, _rule_set_bump) = mpl_token_auth_rules::pda::find_rule_set_address(
        context.payer.pubkey(),
        "test rule_set".to_string(),
    );

    // Create a `create` instruction.
    let create_ix = CreateOrUpdateBuilder::new()
        .payer(context.payer.pubkey())
        .rule_set_pda(rule_set_addr)
        .build(CreateOrUpdateArgs::V1 {
            serialized_rule_set,
        })
        .unwrap()
        .instruction();

    // Add it to a transaction.
    let create_tx = Transaction::new_signed_with_payer(
        &[create_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    // Process the transaction.
    let err = context
        .banks_client
        .process_transaction(create_tx)
        .await
        .expect_err("Creation should fail");

    // Check that error is what we expect.
    assert_custom_error!(err, RuleSetError::RuleSetOwnerMismatch);
}

#[tokio::test]
async fn create_rule_set_buffer_with_different_name_fails() {
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

    // Find buffer PDA.
    let (buffer_pda, _buffer_bump) =
        mpl_token_auth_rules::pda::find_buffer_address(context.payer.pubkey());

    // Write `RuleSet` to buffer.
    let mut overwrite = true;
    for serialized_rule_set_chunk in serialized_rule_set.chunks(1000) {
        // Create a `write_to_buffer` instruction.
        let write_to_buffer_ix = WriteToBufferBuilder::new()
            .payer(context.payer.pubkey())
            .buffer_pda(buffer_pda)
            .build(WriteToBufferArgs::V1 {
                serialized_rule_set: serialized_rule_set_chunk.to_vec(),
                overwrite,
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
        context
            .banks_client
            .process_transaction(write_to_buffer_tx)
            .await
            .unwrap();

        if overwrite {
            overwrite = false;
        }
    }

    // --------------------------------
    // Fail on-chain creation
    // --------------------------------
    // Find RuleSet PDA with DIFFERENT NAME.
    let (rule_set_addr, _rule_set_bump) = mpl_token_auth_rules::pda::find_rule_set_address(
        context.payer.pubkey(),
        "DIFFERENT NAME".to_string(),
    );

    // Create a `create` instruction.
    let create_ix = CreateOrUpdateBuilder::new()
        .payer(context.payer.pubkey())
        .rule_set_pda(rule_set_addr)
        .buffer_pda(buffer_pda)
        .build(CreateOrUpdateArgs::V1 {
            serialized_rule_set: vec![],
        })
        .unwrap()
        .instruction();

    // Add it to a transaction.
    let create_tx = Transaction::new_signed_with_payer(
        &[create_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    // Process the transaction.
    let err = context
        .banks_client
        .process_transaction(create_tx)
        .await
        .expect_err("Creation should fail");

    // Check that error is what we expect.
    assert_custom_error!(err, RuleSetError::DerivedKeyInvalid);
}

#[tokio::test]
async fn create_rule_set_to_wallet_fails() {
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

    // --------------------------------
    // Fail on-chain creation
    // --------------------------------
    // Create a `create` instruction.
    let create_ix = CreateOrUpdateBuilder::new()
        .payer(context.payer.pubkey())
        .rule_set_pda(Keypair::new().pubkey())
        .build(CreateOrUpdateArgs::V1 {
            serialized_rule_set,
        })
        .unwrap()
        .instruction();

    // Add it to a transaction.
    let create_tx = Transaction::new_signed_with_payer(
        &[create_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    // Process the transaction.
    let err = context
        .banks_client
        .process_transaction(create_tx)
        .await
        .expect_err("Creation should fail");

    // Check that error is what we expect.
    assert_custom_error!(err, RuleSetError::DerivedKeyInvalid);
}

#[tokio::test]
async fn create_rule_set_to_wrong_pda_fails() {
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

    // --------------------------------
    // Fail on-chain creation
    // --------------------------------
    // Find RuleSet PDA using WRONG NAME for seed.
    let (wrong_rule_set_addr, _rule_set_bump) = mpl_token_auth_rules::pda::find_rule_set_address(
        context.payer.pubkey(),
        "WRONG NAME".to_string(),
    );

    // Create a `create` instruction.
    let create_ix = CreateOrUpdateBuilder::new()
        .payer(context.payer.pubkey())
        .rule_set_pda(wrong_rule_set_addr)
        .build(CreateOrUpdateArgs::V1 {
            serialized_rule_set,
        })
        .unwrap()
        .instruction();

    // Add it to a transaction.
    let create_tx = Transaction::new_signed_with_payer(
        &[create_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    // Process the transaction.
    let err = context
        .banks_client
        .process_transaction(create_tx)
        .await
        .expect_err("Creation should fail");

    // Check that error is what we expect.
    assert_custom_error!(err, RuleSetError::DerivedKeyInvalid);
}
