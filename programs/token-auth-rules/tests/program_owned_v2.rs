#![cfg(feature = "test-bpf")]

pub mod utils;

use mpl_token_auth_rules::{
    error::RuleSetError,
    instruction::{builders::ValidateBuilder, InstructionBuilder, ValidateArgs},
    payload::{Payload, PayloadType},
    state::{ProgramOwned, RuleSetV2},
};
use solana_program_test::tokio;
use solana_sdk::{
    instruction::AccountMeta, signature::Signer, signer::keypair::Keypair, system_instruction,
    transaction::Transaction,
};
use utils::{create_associated_token_account, create_mint, program_test, Operation, PayloadKey};

#[tokio::test]
async fn program_owned_v2() {
    let mut context = program_test().start_with_context().await;

    // --------------------------------
    // Create RuleSet
    // --------------------------------
    // Create a Rule.  The target must be owned by the program ID specified in the Rule.
    let rule = ProgramOwned::serialize(
        PayloadKey::Destination.to_string(),
        mpl_token_auth_rules::ID,
    )
    .unwrap();

    // Create a RuleSet.
    let rule_set = RuleSetV2::serialize(
        context.payer.pubkey(),
        "test rule_set",
        &[Operation::Transfer {
            scenario: utils::TransferScenario::Holder,
        }
        .to_string()],
        &[&rule],
    )
    .unwrap();

    // Put the RuleSet on chain.
    let rule_set_addr =
        create_rule_set_on_chain_serialized!(&mut context, rule_set, "test rule_set".to_string())
            .await;

    // --------------------------------
    // Validate fail prog owned but zero data length
    // --------------------------------
    // Create a Keypair to simulate a token mint address.
    let mint = Keypair::new();

    // Create an empty account owned by mpl-token-auth-rules.
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

    // Get on-chain account.
    let on_chain_account = context
        .banks_client
        .get_account(program_owned_account.pubkey())
        .await
        .unwrap()
        .unwrap();

    // Verify data length is zero.
    assert_eq!(0, on_chain_account.data.len());

    // Verify account ownership.
    assert_eq!(mpl_token_auth_rules::ID, on_chain_account.owner);

    // Store the payload of data to validate against the rule definition.
    let payload = Payload::from([(
        PayloadKey::Destination.to_string(),
        PayloadType::Pubkey(program_owned_account.pubkey()),
    )]);

    // Create a `validate` instruction.
    let validate_ix = ValidateBuilder::new()
        .rule_set_pda(rule_set_addr)
        .mint(mint.pubkey())
        .additional_rule_accounts(vec![AccountMeta::new_readonly(
            program_owned_account.pubkey(),
            false,
        )])
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
    assert_custom_error!(err, RuleSetError::ProgramOwnedCheckFailed);

    // --------------------------------
    // Validate nonzero data but owned by different program
    // --------------------------------
    let owner = Keypair::new();

    // Create an associated token account for the sole purpose of having an account that is owned
    // by a different program than what is in the rule.
    create_mint(
        &mut context,
        &mint,
        &owner.pubkey(),
        Some(&owner.pubkey()),
        0,
    )
    .await
    .unwrap();

    let associated_token_account =
        create_associated_token_account(&mut context, &owner, &mint.pubkey())
            .await
            .unwrap();

    // Get on-chain account.
    let on_chain_account = context
        .banks_client
        .get_account(associated_token_account)
        .await
        .unwrap()
        .unwrap();

    // Account must have nonzero data to count as program-owned.
    assert!(on_chain_account.data.iter().any(|&x| x != 0));

    // Verify account ownership.
    assert_eq!(spl_token::ID, on_chain_account.owner);

    // Store the payload of data to validate against the rule definition.
    let payload = Payload::from([(
        PayloadKey::Destination.to_string(),
        PayloadType::Pubkey(associated_token_account),
    )]);

    let validate_ix = ValidateBuilder::new()
        .rule_set_pda(rule_set_addr)
        .mint(mint.pubkey())
        .additional_rule_accounts(vec![AccountMeta::new_readonly(
            associated_token_account,
            false,
        )])
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

    // Fail to validate operation.
    let err = process_failing_validate_ix!(&mut context, validate_ix, vec![], None).await;

    // Check that error is what we expect.
    assert_custom_error!(err, RuleSetError::ProgramOwnedCheckFailed);

    // --------------------------------
    // Validate pass
    // --------------------------------
    // Our destination key is going to be an account owned by the mpl-token-auth-rules program.
    // Any one will do so for convenience we just use the `RuleSet`.
    let payload = Payload::from([(
        PayloadKey::Destination.to_string(),
        PayloadType::Pubkey(rule_set_addr),
    )]);

    // Get on-chain account.
    let on_chain_account = context
        .banks_client
        .get_account(rule_set_addr)
        .await
        .unwrap()
        .unwrap();

    // Account must have nonzero data to count as program-owned.
    assert!(on_chain_account.data.iter().any(|&x| x != 0));

    // Verify account ownership.
    assert_eq!(mpl_token_auth_rules::ID, on_chain_account.owner);

    // Create a `validate` instruction.
    let validate_ix = ValidateBuilder::new()
        .rule_set_pda(rule_set_addr)
        .mint(mint.pubkey())
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

    // Validate transfer operation.
    process_passing_validate_ix!(&mut context, validate_ix, vec![], None).await;
}
