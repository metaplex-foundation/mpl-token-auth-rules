#![cfg(feature = "test-bpf")]

pub mod utils;

use mpl_token_auth_rules::{
    error::RuleSetError,
    instruction::{builders::ValidateBuilder, InstructionBuilder, ValidateArgs},
    payload::Payload,
    state::{Rule, RuleSetV1},
};

use solana_program::program_error::ProgramError;
use solana_program::system_instruction;
use solana_program_test::{tokio, BanksClientError};
use solana_sdk::{
    signature::Signer,
    signer::keypair::Keypair,
    transaction::{Transaction, TransactionError},
};
use utils::{program_test, Operation};

#[tokio::test]
#[should_panic]
async fn validate_update_rule_state_payer_not_signer_panics() {
    let mut context = program_test().start_with_context().await;

    // Create a RuleSet.
    let mut rule_set = RuleSetV1::new("test rule_set".to_string(), context.payer.pubkey());
    rule_set
        .add(
            Operation::Transfer {
                scenario: utils::TransferScenario::Holder,
            }
            .to_string(),
            Rule::Pass,
        )
        .unwrap();

    // Put the RuleSet on chain.
    let rule_set_addr =
        create_rule_set_on_chain!(&mut context, rule_set, "test rule_set".to_string()).await;

    // Create a Keypair to simulate a token mint address.
    let mint = Keypair::new().pubkey();

    let rule_authority = Keypair::new();

    let (rule_set_state_pda, _rule_set_state_pda_bump) =
        mpl_token_auth_rules::pda::find_rule_set_state_address(
            context.payer.pubkey(),
            "test rule_set".to_string(),
            mint,
        );

    // Create a `validate` instruction with `update_rule_state` set to true.
    let other_payer = Keypair::new();
    let validate_ix = ValidateBuilder::new()
        .rule_set_pda(rule_set_addr)
        .mint(mint)
        .payer(other_payer.pubkey())
        .rule_authority(rule_authority.pubkey())
        .rule_set_state_pda(rule_set_state_pda)
        .additional_rule_accounts(vec![])
        .build(ValidateArgs::V1 {
            operation: Operation::Transfer {
                scenario: utils::TransferScenario::Holder,
            }
            .to_string(),
            payload: Payload::default(),
            update_rule_state: true,
            rule_set_revision: None,
        })
        .unwrap()
        .instruction();

    // Add ix to a transaction.
    let validate_tx = Transaction::new_signed_with_payer(
        &[validate_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &rule_authority],
        context.last_blockhash,
    );

    // Process the transaction.  It will panic because of not enough signers.
    let _result = context.banks_client.process_transaction(validate_tx).await;
}

#[tokio::test]
async fn validate_update_rule_state_payer_not_provided_fails() {
    let mut context = program_test().start_with_context().await;

    // Create a RuleSet.
    let mut rule_set = RuleSetV1::new("test rule_set".to_string(), context.payer.pubkey());
    rule_set
        .add(
            Operation::Transfer {
                scenario: utils::TransferScenario::Holder,
            }
            .to_string(),
            Rule::Pass,
        )
        .unwrap();

    // Put the RuleSet on chain.
    let rule_set_addr =
        create_rule_set_on_chain!(&mut context, rule_set, "test rule_set".to_string()).await;

    // Create a Keypair to simulate a token mint address.
    let mint = Keypair::new().pubkey();

    let rule_authority = Keypair::new();

    let (rule_set_state_pda, _rule_set_state_pda_bump) =
        mpl_token_auth_rules::pda::find_rule_set_state_address(
            context.payer.pubkey(),
            "test rule_set".to_string(),
            mint,
        );

    // Create a `validate` instruction with `update_rule_state` set to true.
    let validate_ix = ValidateBuilder::new()
        .rule_set_pda(rule_set_addr)
        .mint(mint)
        .rule_authority(rule_authority.pubkey())
        .rule_set_state_pda(rule_set_state_pda)
        .additional_rule_accounts(vec![])
        .build(ValidateArgs::V1 {
            operation: Operation::Transfer {
                scenario: utils::TransferScenario::Holder,
            }
            .to_string(),
            payload: Payload::default(),
            update_rule_state: true,
            rule_set_revision: None,
        })
        .unwrap()
        .instruction();

    // Fail to validate operation.
    let err =
        process_failing_validate_ix!(&mut context, validate_ix, vec![&rule_authority], None).await;

    // Deconstruct the error code and make sure it is what we expect.
    match err {
        BanksClientError::TransactionError(TransactionError::InstructionError(0, err)) => {
            assert_eq!(
                ProgramError::try_from(err).unwrap_or_else(|_| panic!(
                    "Could not convert InstructionError to ProgramError",
                )),
                ProgramError::NotEnoughAccountKeys,
            );
        }
        _ => panic!("Unexpected error: {}", err),
    }
}

#[tokio::test]
async fn validate_rule_set_with_wallet_fails() {
    let mut context = program_test().start_with_context().await;

    // Create a Keypair to simulate a token mint address.
    let mint = Keypair::new().pubkey();

    // Create a `validate` instruction.
    let validate_ix = ValidateBuilder::new()
        .rule_set_pda(Keypair::new().pubkey())
        .mint(mint)
        .additional_rule_accounts(vec![])
        .build(ValidateArgs::V1 {
            operation: Operation::Transfer {
                scenario: utils::TransferScenario::Holder,
            }
            .to_string(),
            payload: Payload::default(),
            update_rule_state: false,
            rule_set_revision: None,
        })
        .unwrap()
        .instruction();

    // Fail to validate operation.
    let err = process_failing_validate_ix!(&mut context, validate_ix, vec![], None).await;

    // Check that error is what we expect.
    assert_custom_error!(err, RuleSetError::IncorrectOwner);
}

#[tokio::test]
async fn validate_rule_set_with_uninitialized_pda_fails() {
    let mut context = program_test().start_with_context().await;

    // Find RuleSet PDA.
    let (rule_set_addr, _rule_set_bump) = mpl_token_auth_rules::pda::find_rule_set_address(
        context.payer.pubkey(),
        "test rule_set".to_string(),
    );

    // Create a Keypair to simulate a token mint address.
    let mint = Keypair::new().pubkey();

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
            payload: Payload::default(),
            update_rule_state: false,
            rule_set_revision: None,
        })
        .unwrap()
        .instruction();

    // Fail to validate operation.
    let err = process_failing_validate_ix!(&mut context, validate_ix, vec![], None).await;

    // Check that error is what we expect.
    assert_custom_error!(err, RuleSetError::IncorrectOwner);
}

#[tokio::test]
async fn validate_rule_set_with_zero_data_fails() {
    let mut context = program_test().start_with_context().await;

    // Create an account owned by mpl-token-auth-rules.  This isn't a PDA but we expect to fail
    // before the derivation check because the data length is zero.
    let program_owned_account = Keypair::new();
    let rent = context.banks_client.get_rent().await.unwrap();
    let tx = Transaction::new_signed_with_payer(
        &[system_instruction::create_account(
            &context.payer.pubkey(),
            &program_owned_account.pubkey(),
            rent.minimum_balance(0),
            0,
            &mpl_token_auth_rules::ID,
        )],
        Some(&context.payer.pubkey()),
        &[&context.payer, &program_owned_account],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await.unwrap();

    // Create a Keypair to simulate a token mint address.
    let mint = Keypair::new().pubkey();

    // Create a `validate` instruction.
    let validate_ix = ValidateBuilder::new()
        .rule_set_pda(program_owned_account.pubkey())
        .mint(mint)
        .additional_rule_accounts(vec![])
        .build(ValidateArgs::V1 {
            operation: Operation::Transfer {
                scenario: utils::TransferScenario::Holder,
            }
            .to_string(),
            payload: Payload::default(),
            update_rule_state: false,
            rule_set_revision: None,
        })
        .unwrap()
        .instruction();

    // Fail to validate operation.
    let err = process_failing_validate_ix!(&mut context, validate_ix, vec![], None).await;

    // Check that error is what we expect.
    assert_custom_error!(err, RuleSetError::DataIsEmpty);
}

#[tokio::test]
async fn validate_rule_set_with_incorrect_data_fails() {
    let mut context = program_test().start_with_context().await;

    // Create an account owned by mpl-token-auth-rules.  This isn't a PDA but we expect to fail
    // before the derivation check because the data will not deserialize properly into a `RuleSet`.
    // The deserialization is done in the processor before the derivation check because the
    // derivation uses the `RuleSet` name and owner from the deserialized data as seeds.
    let program_owned_account = Keypair::new();
    let rent = context.banks_client.get_rent().await.unwrap();
    let tx = Transaction::new_signed_with_payer(
        &[system_instruction::create_account(
            &context.payer.pubkey(),
            &program_owned_account.pubkey(),
            rent.minimum_balance(1000),
            1000,
            &mpl_token_auth_rules::ID,
        )],
        Some(&context.payer.pubkey()),
        &[&context.payer, &program_owned_account],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await.unwrap();

    // Create a Keypair to simulate a token mint address.
    let mint = Keypair::new().pubkey();

    // Create a `validate` instruction.
    let validate_ix = ValidateBuilder::new()
        .rule_set_pda(program_owned_account.pubkey())
        .mint(mint)
        .additional_rule_accounts(vec![])
        .build(ValidateArgs::V1 {
            operation: Operation::Transfer {
                scenario: utils::TransferScenario::Holder,
            }
            .to_string(),
            payload: Payload::default(),
            update_rule_state: false,
            rule_set_revision: None,
        })
        .unwrap()
        .instruction();

    // Fail to validate operation.
    let err = process_failing_validate_ix!(&mut context, validate_ix, vec![], None).await;

    // Check that error is what we expect.  This happens to be how data with all zeros fails to
    // deserialize.
    assert_custom_error!(err, RuleSetError::UnsupportedRuleSetRevMapVersion);
}

#[tokio::test]
async fn validate_update_rule_state_wrong_state_pda_fails() {
    let mut context = program_test().start_with_context().await;

    // Create a RuleSet.
    let mut rule_set = RuleSetV1::new("test rule_set".to_string(), context.payer.pubkey());
    rule_set
        .add(
            Operation::Transfer {
                scenario: utils::TransferScenario::Holder,
            }
            .to_string(),
            Rule::Pass,
        )
        .unwrap();

    // Put the RuleSet on chain.
    let rule_set_addr =
        create_rule_set_on_chain!(&mut context, rule_set, "test rule_set".to_string()).await;

    // Create a Keypair to simulate a token mint address.
    let mint = Keypair::new().pubkey();

    let rule_authority = Keypair::new();

    // Find RuleSet state PDA using WRONG NAME for seed.
    let (rule_set_state_pda, _rule_set_state_pda_bump) =
        mpl_token_auth_rules::pda::find_rule_set_state_address(
            context.payer.pubkey(),
            "WRONG NAME".to_string(),
            mint,
        );

    // Create a `validate` instruction with `update_rule_state` set to true.
    let validate_ix = ValidateBuilder::new()
        .rule_set_pda(rule_set_addr)
        .mint(mint)
        .payer(context.payer.pubkey())
        .rule_authority(rule_authority.pubkey())
        .rule_set_state_pda(rule_set_state_pda)
        .additional_rule_accounts(vec![])
        .build(ValidateArgs::V1 {
            operation: Operation::Transfer {
                scenario: utils::TransferScenario::Holder,
            }
            .to_string(),
            payload: Payload::default(),
            update_rule_state: true,
            rule_set_revision: None,
        })
        .unwrap()
        .instruction();

    // Fail to validate operation.
    let err =
        process_failing_validate_ix!(&mut context, validate_ix, vec![&rule_authority], None).await;

    // Check that error is what we expect.
    assert_custom_error!(err, RuleSetError::DerivedKeyInvalid);
}

#[tokio::test]
async fn validate_update_rule_state_state_pda_not_provided_fails() {
    let mut context = program_test().start_with_context().await;

    // Create a RuleSet.
    let mut rule_set = RuleSetV1::new("test rule_set".to_string(), context.payer.pubkey());
    rule_set
        .add(
            Operation::Transfer {
                scenario: utils::TransferScenario::Holder,
            }
            .to_string(),
            Rule::Pass,
        )
        .unwrap();

    // Put the RuleSet on chain.
    let rule_set_addr =
        create_rule_set_on_chain!(&mut context, rule_set, "test rule_set".to_string()).await;

    // Create a Keypair to simulate a token mint address.
    let mint = Keypair::new().pubkey();

    let rule_authority = Keypair::new();

    // Create a `validate` instruction with `update_rule_state` set to true.
    let validate_ix = ValidateBuilder::new()
        .rule_set_pda(rule_set_addr)
        .mint(mint)
        .payer(context.payer.pubkey())
        .rule_authority(rule_authority.pubkey())
        .additional_rule_accounts(vec![])
        .build(ValidateArgs::V1 {
            operation: Operation::Transfer {
                scenario: utils::TransferScenario::Holder,
            }
            .to_string(),
            payload: Payload::default(),
            update_rule_state: true,
            rule_set_revision: None,
        })
        .unwrap()
        .instruction();

    // Fail to validate operation.
    let err =
        process_failing_validate_ix!(&mut context, validate_ix, vec![&rule_authority], None).await;

    // Deconstruct the error code and make sure it is what we expect.
    match err {
        BanksClientError::TransactionError(TransactionError::InstructionError(0, err)) => {
            assert_eq!(
                ProgramError::try_from(err).unwrap_or_else(|_| panic!(
                    "Could not convert InstructionError to ProgramError",
                )),
                ProgramError::NotEnoughAccountKeys,
            );
        }
        _ => panic!("Unexpected error: {}", err),
    }
}

#[tokio::test]
async fn validate_update_rule_state_incorrect_auth() {
    let mut context = program_test().start_with_context().await;

    // Create a Rule that uses Rule Authority.
    let rule_authority = Keypair::new();
    let rule = Rule::Frequency {
        authority: rule_authority.pubkey(),
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

    // Put the RuleSet on chain.
    let rule_set_addr =
        create_rule_set_on_chain!(&mut context, rule_set, "test rule_set".to_string()).await;

    // Create a Keypair to simulate a token mint address.
    let mint = Keypair::new().pubkey();

    let (rule_set_state_pda, _rule_set_state_pda_bump) =
        mpl_token_auth_rules::pda::find_rule_set_state_address(
            context.payer.pubkey(),
            "test rule_set".to_string(),
            mint,
        );

    // Create a `validate` instruction with `update_rule_state` set to true.
    let validate_ix = ValidateBuilder::new()
        .rule_set_pda(rule_set_addr)
        .mint(mint)
        .payer(context.payer.pubkey())
        .rule_authority(context.payer.pubkey())
        .rule_set_state_pda(rule_set_state_pda)
        .additional_rule_accounts(vec![])
        .build(ValidateArgs::V1 {
            operation: Operation::Transfer {
                scenario: utils::TransferScenario::Holder,
            }
            .to_string(),
            payload: Payload::default(),
            update_rule_state: true,
            rule_set_revision: None,
        })
        .unwrap()
        .instruction();

    // Fail to validate operation.
    let err = process_failing_validate_ix!(&mut context, validate_ix, vec![], None).await;

    // Check that error is what we expect.
    assert_custom_error!(err, RuleSetError::RuleAuthorityIsNotSigner);
}

#[tokio::test]
async fn validate_update_rule_state_missing_auth() {
    let mut context = program_test().start_with_context().await;

    // Create a Rule that uses Rule Authority.
    let rule_authority = Keypair::new();
    let rule = Rule::Frequency {
        authority: rule_authority.pubkey(),
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

    // Put the RuleSet on chain.
    let rule_set_addr =
        create_rule_set_on_chain!(&mut context, rule_set, "test rule_set".to_string()).await;

    // Create a Keypair to simulate a token mint address.
    let mint = Keypair::new().pubkey();

    let (rule_set_state_pda, _rule_set_state_pda_bump) =
        mpl_token_auth_rules::pda::find_rule_set_state_address(
            context.payer.pubkey(),
            "test rule_set".to_string(),
            mint,
        );

    // Create a `validate` instruction with `update_rule_state` set to true.
    let validate_ix = ValidateBuilder::new()
        .rule_set_pda(rule_set_addr)
        .mint(mint)
        .payer(context.payer.pubkey())
        .rule_set_state_pda(rule_set_state_pda)
        .additional_rule_accounts(vec![])
        .build(ValidateArgs::V1 {
            operation: Operation::Transfer {
                scenario: utils::TransferScenario::Holder,
            }
            .to_string(),
            payload: Payload::default(),
            update_rule_state: true,
            rule_set_revision: None,
        })
        .unwrap()
        .instruction();

    // Fail to validate operation.
    let err = process_failing_validate_ix!(&mut context, validate_ix, vec![], None).await;

    // Check that error is what we expect.
    assert_custom_error!(err, RuleSetError::MissingAccount);
}
