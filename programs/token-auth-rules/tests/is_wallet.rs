#![cfg(feature = "test-bpf")]

pub mod utils;

use mpl_token_auth_rules::{
    error::RuleSetError,
    instruction::{builders::ValidateBuilder, InstructionBuilder, ValidateArgs},
    payload::{Payload, PayloadType},
    state::{Rule, RuleSetV1},
};
use solana_program_test::tokio;
use solana_sdk::{instruction::AccountMeta, signature::Signer, signer::keypair::Keypair};
use utils::{program_test, Operation, PayloadKey};

#[tokio::test]
async fn is_wallet() {
    let mut context = program_test().start_with_context().await;

    // --------------------------------
    // Create RuleSet
    // --------------------------------
    // Create a Rule.
    let rule = Rule::IsWallet {
        field: PayloadKey::Source.to_string(),
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

    // --------------------------------
    // Validate not implemented
    // (this will become pass later)
    // --------------------------------
    // Keypair to check.
    let wallet = Keypair::new();

    // Create a Keypair to simulate a token mint address.
    let mint = Keypair::new().pubkey();

    let payload = Payload::from([(
        PayloadKey::Source.to_string(),
        PayloadType::Pubkey(wallet.pubkey()),
    )]);

    // Create a `validate` instruction.
    let validate_ix = ValidateBuilder::new()
        .rule_set_pda(rule_set_addr)
        .mint(mint)
        .additional_rule_accounts(vec![AccountMeta::new_readonly(wallet.pubkey(), false)])
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
    assert_custom_error!(err, RuleSetError::NotImplemented);
}
