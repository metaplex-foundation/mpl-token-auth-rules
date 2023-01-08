#![cfg(feature = "test-bpf")]

pub mod utils;

use mpl_token_auth_rules::{
    error::RuleSetError,
    instruction::{builders::ValidateBuilder, InstructionBuilder, ValidateArgs},
    payload::{Payload, PayloadType},
    state::{Rule, RuleSet},
};
use solana_program_test::tokio;
use solana_sdk::{
    instruction::AccountMeta, signature::Signer, signer::keypair::Keypair, system_instruction,
    transaction::Transaction,
};
use utils::{create_rule_set_on_chain, program_test, Operation, PayloadKey};

#[tokio::test]
async fn program_owned() {
    let mut context = program_test().start_with_context().await;

    // --------------------------------
    // Create RuleSet
    // --------------------------------
    // Create a Rule.  The target must be owned by the program ID specified in the Rule.
    let rule = Rule::ProgramOwned {
        program: mpl_token_metadata::id(),
        field: PayloadKey::Destination.to_string(),
    };

    // Create a RuleSet.
    let mut rule_set = RuleSet::new("test rule_set".to_string(), context.payer.pubkey());
    rule_set
        .add(Operation::OwnerTransfer.to_string(), rule)
        .unwrap();

    println!("{:#?}", rule_set);

    // Put the RuleSet on chain.
    let rule_set_addr =
        create_rule_set_on_chain(&mut context, rule_set, "test rule_set".to_string()).await;

    // --------------------------------
    // Validate fail
    // --------------------------------
    // Create a Keypair to simulate a token mint address.
    let mint = Keypair::new().pubkey();

    // Store the payload of data to validate against the rule definition.
    // In this case the Target will be used to look up the `AccountInfo`
    // and see who the owner is.  Here we put in the WRONG Pubkey.
    let wrong_account = Keypair::new();
    let payload = Payload::from([(
        PayloadKey::Destination.to_string(),
        PayloadType::Pubkey(wrong_account.pubkey()),
    )]);

    // We also pass the WRONG account as an additional rule account.
    // It will be found by the Rule but owner will be wrong.
    let validate_ix = ValidateBuilder::new()
        .rule_set_pda(rule_set_addr)
        .mint(mint)
        .additional_rule_accounts(vec![AccountMeta::new_readonly(
            wrong_account.pubkey(),
            false,
        )])
        .build(ValidateArgs::V1 {
            operation: Operation::OwnerTransfer.to_string(),
            payload,
            update_rule_state: false,
        })
        .unwrap()
        .instruction();

    // Fail to validate Transfer operation.
    let err = process_failing_validate_ix!(&mut context, validate_ix, vec![]).await;

    // Check that error is what we expect.
    assert_rule_set_error!(err, RuleSetError::ProgramOwnedCheckFailed);

    // --------------------------------
    // Validate pass
    // --------------------------------
    // Create an account owned mpl-token-metadata.
    let program_owned_account = Keypair::new();
    let rent = context.banks_client.get_rent().await.unwrap();
    let tx = Transaction::new_signed_with_payer(
        &[system_instruction::create_account(
            &context.payer.pubkey(),
            &program_owned_account.pubkey(),
            rent.minimum_balance(0),
            0,
            &mpl_token_metadata::id(),
        )],
        Some(&context.payer.pubkey()),
        &[&context.payer, &program_owned_account],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await.unwrap();

    // This time put the CORRECT Pubkey into the Payload and the validate instruction.
    let payload = Payload::from([(
        PayloadKey::Destination.to_string(),
        PayloadType::Pubkey(program_owned_account.pubkey()),
    )]);

    // Create a `validate` instruction.
    let validate_ix = ValidateBuilder::new()
        .rule_set_pda(rule_set_addr)
        .mint(mint)
        .additional_rule_accounts(vec![AccountMeta::new_readonly(
            program_owned_account.pubkey(),
            false,
        )])
        .build(ValidateArgs::V1 {
            operation: Operation::OwnerTransfer.to_string(),
            payload,
            update_rule_state: false,
        })
        .unwrap()
        .instruction();

    // Validate Transfer operation.
    process_passing_validate_ix!(&mut context, validate_ix, vec![]).await;
}
