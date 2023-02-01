#![cfg(feature = "test-bpf")]

pub mod utils;

use mpl_token_auth_rules::{
    error::RuleSetError,
    instruction::{builders::ValidateBuilder, InstructionBuilder, ValidateArgs},
    payload::{Payload, PayloadType},
    state::{CompareOp, Rule, RuleSetV1},
};
use solana_program::pubkey::Pubkey;
use solana_program_test::{tokio, ProgramTestContext};
use solana_sdk::{
    instruction::AccountMeta, signature::Signer, signer::keypair::Keypair, system_instruction,
    transaction::Transaction,
};
use utils::{
    create_associated_token_account, create_mint, program_test, Operation, PayloadKey,
    TransferScenario,
};

static PROGRAM_ALLOW_LIST: [Pubkey; 1] = [mpl_token_auth_rules::ID];

macro_rules! get_primitive_rules {
    (
        $nft_amount:ident,
        $source_program_allow_list:ident,
        $dest_program_allow_list:ident,
        $source_is_wallet:ident,
        $dest_is_wallet:ident
    ) => {
        let $nft_amount = Rule::Amount {
            field: PayloadKey::Amount.to_string(),
            amount: 1,
            operator: CompareOp::Eq,
        };

        let $source_program_allow_list = Rule::ProgramOwnedList {
            programs: PROGRAM_ALLOW_LIST.to_vec(),
            field: PayloadKey::Source.to_string(),
        };

        let $dest_program_allow_list = Rule::ProgramOwnedList {
            programs: PROGRAM_ALLOW_LIST.to_vec(),
            field: PayloadKey::Destination.to_string(),
        };

        let $source_is_wallet = Rule::IsWallet {
            field: PayloadKey::Source.to_string(),
        };

        let $dest_is_wallet = Rule::IsWallet {
            field: PayloadKey::Destination.to_string(),
        };
    };
}

fn get_rules() -> (Rule, Rule) {
    get_primitive_rules!(
        nft_amount,
        source_program_allow_list,
        dest_program_allow_list,
        source_is_wallet,
        dest_is_wallet
    );

    // --------------------------------
    // Create Rules
    // --------------------------------
    // amount is 1 && (source is on allow list || dest is on allow list)
    let transfer_rule = Rule::All {
        rules: vec![
            nft_amount.clone(),
            Rule::Any {
                rules: vec![source_program_allow_list, dest_program_allow_list],
            },
        ],
    };

    // (amount is 1 && source is wallet && dest is wallet)
    let wallet_to_wallet_rule = Rule::All {
        rules: vec![nft_amount, source_is_wallet, dest_is_wallet],
    };

    (transfer_rule, wallet_to_wallet_rule)
}

const RULE_SET_NAME: &str = "Metaplex Royalty RuleSet Dev";

// Compose operations with scenarios.
const OWNER_OPERATION: Operation = Operation::Transfer {
    scenario: TransferScenario::Holder,
};

const TRANSFER_DELEGATE_OPERATION: Operation = Operation::Transfer {
    scenario: TransferScenario::TransferDelegate,
};

const SALE_DELEGATE_OPERATION: Operation = Operation::Transfer {
    scenario: TransferScenario::SaleDelegate,
};

const MIGRATION_DELEGATE_OPERATION: Operation = Operation::Transfer {
    scenario: TransferScenario::MigrationDelegate,
};

const WALLET_TO_WALLET_OPERATION: Operation = Operation::Transfer {
    scenario: TransferScenario::WalletToWallet,
};

async fn create_royalty_rule_set(context: &mut ProgramTestContext) -> Pubkey {
    // Create RuleSet
    let (transfer_rule, wallet_to_wallet_rule) = get_rules();
    let mut royalty_rule_set = RuleSetV1::new(RULE_SET_NAME.to_string(), context.payer.pubkey());

    // Add operations to `RuleSet`.
    royalty_rule_set
        .add(OWNER_OPERATION.to_string(), transfer_rule.clone())
        .unwrap();
    royalty_rule_set
        .add(
            TRANSFER_DELEGATE_OPERATION.to_string(),
            transfer_rule.clone(),
        )
        .unwrap();
    royalty_rule_set
        .add(SALE_DELEGATE_OPERATION.to_string(), transfer_rule.clone())
        .unwrap();
    royalty_rule_set
        .add(MIGRATION_DELEGATE_OPERATION.to_string(), transfer_rule)
        .unwrap();
    royalty_rule_set
        .add(
            WALLET_TO_WALLET_OPERATION.to_string(),
            wallet_to_wallet_rule,
        )
        .unwrap();

    println!("{:#?}", royalty_rule_set);

    // Put the `RuleSet` on chain.
    create_big_rule_set_on_chain!(context, royalty_rule_set.clone(), RULE_SET_NAME.to_string())
        .await
}

#[tokio::test]
async fn create_rule_set() {
    let mut context = program_test().start_with_context().await;
    let _rule_set_addr = create_royalty_rule_set(&mut context).await;
}

#[tokio::test]
async fn wallet_to_wallet_unimplemented() {
    let mut context = program_test().start_with_context().await;
    let rule_set_addr = create_royalty_rule_set(&mut context).await;

    // Create a Keypair to simulate a token mint address.
    let mint = Keypair::new();

    // Create source and destination wallets.
    let source = Keypair::new();
    let dest = Keypair::new();

    // Store the payload of data to validate against the rule definition.
    let payload = Payload::from([
        (PayloadKey::Amount.to_string(), PayloadType::Number(1)),
        (
            PayloadKey::Source.to_string(),
            PayloadType::Pubkey(source.pubkey()),
        ),
        (
            PayloadKey::Destination.to_string(),
            PayloadType::Pubkey(dest.pubkey()),
        ),
    ]);

    // Create a `validate` instruction.
    let validate_ix = ValidateBuilder::new()
        .rule_set_pda(rule_set_addr)
        .mint(mint.pubkey())
        .additional_rule_accounts(vec![
            AccountMeta::new_readonly(source.pubkey(), false),
            AccountMeta::new_readonly(dest.pubkey(), false),
        ])
        .build(ValidateArgs::V1 {
            operation: WALLET_TO_WALLET_OPERATION.to_string(),
            payload,
            update_rule_state: false,
            rule_set_revision: None,
        })
        .unwrap()
        .instruction();

    // Validate fail operation.
    let err = process_failing_validate_ix!(&mut context, validate_ix, vec![], None).await;

    // Check that error is what we expect.  The `IsWallet` rule currently returns `NotImplemented`.
    assert_custom_error!(err, RuleSetError::NotImplemented);
}

#[tokio::test]
async fn wallet_to_prog_owned() {
    let mut context = program_test().start_with_context().await;
    let rule_set_addr = create_royalty_rule_set(&mut context).await;

    // Create a Keypair to simulate a token mint address.
    let mint = Keypair::new();

    // Source key is a wallet.
    let source = Keypair::new();

    // Our destination key is going to be an account owned by the mpl-token-auth-rules program.
    // Any one will do so for convenience we just use the RuleSet.
    let payload = Payload::from([
        (PayloadKey::Amount.to_string(), PayloadType::Number(1)),
        (
            PayloadKey::Source.to_string(),
            PayloadType::Pubkey(source.pubkey()),
        ),
        (
            PayloadKey::Destination.to_string(),
            PayloadType::Pubkey(rule_set_addr),
        ),
    ]);

    // Create a `validate` instruction.
    let validate_ix = ValidateBuilder::new()
        .rule_set_pda(rule_set_addr)
        .mint(mint.pubkey())
        .additional_rule_accounts(vec![
            AccountMeta::new_readonly(source.pubkey(), false),
            AccountMeta::new_readonly(rule_set_addr, false),
        ])
        .build(ValidateArgs::V1 {
            operation: OWNER_OPERATION.to_string(),
            payload,
            update_rule_state: false,
            rule_set_revision: None,
        })
        .unwrap()
        .instruction();

    // Validate operation.
    process_passing_validate_ix!(&mut context, validate_ix, vec![], None).await;
}

#[tokio::test]
async fn prog_owned_to_prog_owned() {
    let mut context = program_test().start_with_context().await;
    let rule_set_addr = create_royalty_rule_set(&mut context).await;

    // Create a Keypair to simulate a token mint address.
    let mint = Keypair::new();

    // Our source and destination keys are going to be accounts owned by the mpl-token-auth-rules
    // program.  Any one will do so for convenience we just use two `RuleSets`.
    let second_rule_set = RuleSetV1::new("second_rule_set".to_string(), context.payer.pubkey());

    let second_rule_set_addr =
        create_rule_set_on_chain!(&mut context, second_rule_set, "second_rule_set".to_string())
            .await;

    // Store the payload of data to validate against the rule definition.
    let payload = Payload::from([
        (PayloadKey::Amount.to_string(), PayloadType::Number(1)),
        (
            PayloadKey::Source.to_string(),
            PayloadType::Pubkey(rule_set_addr),
        ),
        (
            PayloadKey::Destination.to_string(),
            PayloadType::Pubkey(second_rule_set_addr),
        ),
    ]);

    // Create a `validate` instruction.
    let validate_ix = ValidateBuilder::new()
        .rule_set_pda(rule_set_addr)
        .mint(mint.pubkey())
        .additional_rule_accounts(vec![
            AccountMeta::new_readonly(rule_set_addr, false),
            AccountMeta::new_readonly(second_rule_set_addr, false),
        ])
        .build(ValidateArgs::V1 {
            operation: TRANSFER_DELEGATE_OPERATION.to_string(),
            payload,
            update_rule_state: false,
            rule_set_revision: None,
        })
        .unwrap()
        .instruction();

    // Validate operation.
    process_passing_validate_ix!(&mut context, validate_ix, vec![], None).await;
}

#[tokio::test]
async fn prog_owned_to_wallet() {
    let mut context = program_test().start_with_context().await;
    let rule_set_addr = create_royalty_rule_set(&mut context).await;

    // Create a Keypair to simulate a token mint address.
    let mint = Keypair::new();

    // Destination key is a wallet.
    let dest = Keypair::new();

    // Our source key is going to be an account owned by the mpl-token-auth-rules program.  Any one
    // will do so for convenience we just use the `RuleSet`.
    let payload = Payload::from([
        (PayloadKey::Amount.to_string(), PayloadType::Number(1)),
        (
            PayloadKey::Source.to_string(),
            PayloadType::Pubkey(rule_set_addr),
        ),
        (
            PayloadKey::Destination.to_string(),
            PayloadType::Pubkey(dest.pubkey()),
        ),
    ]);

    // Create a `validate` instruction.
    let validate_ix = ValidateBuilder::new()
        .rule_set_pda(rule_set_addr)
        .mint(mint.pubkey())
        .additional_rule_accounts(vec![
            AccountMeta::new_readonly(rule_set_addr, false),
            AccountMeta::new_readonly(dest.pubkey(), false),
        ])
        .build(ValidateArgs::V1 {
            operation: SALE_DELEGATE_OPERATION.to_string(),
            payload,
            update_rule_state: false,
            rule_set_revision: None,
        })
        .unwrap()
        .instruction();

    // Validate operation.
    process_passing_validate_ix!(&mut context, validate_ix, vec![], None).await;
}

#[tokio::test]
async fn wrong_amount_fails() {
    let mut context = program_test().start_with_context().await;
    let rule_set_addr = create_royalty_rule_set(&mut context).await;

    // Create a Keypair to simulate a token mint address.
    let mint = Keypair::new();

    // Destination key is a wallet.
    let dest = Keypair::new();

    // Store the payload of data to validate against the rule definition, using the WRONG amount.
    // Our source key is going to be an account owned by the mpl-token-auth-rules program.  Any one
    // will do so for convenience we just use the `RuleSet`.
    let payload = Payload::from([
        (PayloadKey::Amount.to_string(), PayloadType::Number(2)),
        (
            PayloadKey::Source.to_string(),
            PayloadType::Pubkey(rule_set_addr),
        ),
        (
            PayloadKey::Destination.to_string(),
            PayloadType::Pubkey(dest.pubkey()),
        ),
    ]);

    // Create a `validate` instruction.
    let validate_ix = ValidateBuilder::new()
        .rule_set_pda(rule_set_addr)
        .mint(mint.pubkey())
        .additional_rule_accounts(vec![
            AccountMeta::new_readonly(rule_set_addr, false),
            AccountMeta::new_readonly(dest.pubkey(), false),
        ])
        .build(ValidateArgs::V1 {
            operation: SALE_DELEGATE_OPERATION.to_string(),
            payload,
            update_rule_state: false,
            rule_set_revision: None,
        })
        .unwrap()
        .instruction();

    // Fail to validate operation.
    let err = process_failing_validate_ix!(&mut context, validate_ix, vec![], None).await;

    // Check that error is what we expect.  Amount was greater than that allowed in the rule so it
    // failed.
    assert_custom_error!(err, RuleSetError::AmountCheckFailed);
}

#[tokio::test]
async fn prog_owner_not_on_list_fails() {
    let mut context = program_test().start_with_context().await;
    let rule_set_addr = create_royalty_rule_set(&mut context).await;

    // Create a Keypair to simulate a token mint address.
    let mint = Keypair::new();

    // Source key is a wallet.
    let source = Keypair::new();

    // Create an associated token account for the sole purpose of having an account that is owned
    // by a different program than what is in the rule.
    create_mint(
        &mut context,
        &mint,
        &source.pubkey(),
        Some(&source.pubkey()),
        0,
    )
    .await
    .unwrap();

    let associated_token_account =
        create_associated_token_account(&mut context, &source, &mint.pubkey())
            .await
            .unwrap();

    // Store the payload of data to validate against the rule definition.
    let payload = Payload::from([
        (PayloadKey::Amount.to_string(), PayloadType::Number(1)),
        (
            PayloadKey::Source.to_string(),
            PayloadType::Pubkey(source.pubkey()),
        ),
        (
            PayloadKey::Destination.to_string(),
            PayloadType::Pubkey(associated_token_account),
        ),
    ]);

    // Create a `validate` instruction.
    let validate_ix = ValidateBuilder::new()
        .rule_set_pda(rule_set_addr)
        .mint(mint.pubkey())
        .additional_rule_accounts(vec![
            AccountMeta::new_readonly(source.pubkey(), false),
            AccountMeta::new_readonly(associated_token_account, false),
        ])
        .build(ValidateArgs::V1 {
            operation: OWNER_OPERATION.to_string(),
            payload,
            update_rule_state: false,
            rule_set_revision: None,
        })
        .unwrap()
        .instruction();

    // Fail to validate operation.
    let err = process_failing_validate_ix!(&mut context, validate_ix, vec![], None).await;

    // Check that error is what we expect.  Program owner was not on the allow list.
    assert_custom_error!(err, RuleSetError::ProgramOwnedListCheckFailed);
}

#[tokio::test]
async fn prog_owned_but_zero_data_length() {
    let mut context = program_test().start_with_context().await;
    let rule_set_addr = create_royalty_rule_set(&mut context).await;

    // Create a Keypair to simulate a token mint address.
    let mint = Keypair::new();

    // Source key is a wallet.
    let source = Keypair::new();

    // Create an account owned by mpl-token-auth-rules.
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

    // Store the payload of data to validate against the rule definition.
    let payload = Payload::from([
        (PayloadKey::Amount.to_string(), PayloadType::Number(1)),
        (
            PayloadKey::Source.to_string(),
            PayloadType::Pubkey(source.pubkey()),
        ),
        (
            PayloadKey::Destination.to_string(),
            PayloadType::Pubkey(program_owned_account.pubkey()),
        ),
    ]);

    // Create a `validate` instruction.
    let validate_ix = ValidateBuilder::new()
        .rule_set_pda(rule_set_addr)
        .mint(mint.pubkey())
        .additional_rule_accounts(vec![
            AccountMeta::new_readonly(source.pubkey(), false),
            AccountMeta::new_readonly(program_owned_account.pubkey(), false),
        ])
        .build(ValidateArgs::V1 {
            operation: OWNER_OPERATION.to_string(),
            payload,
            update_rule_state: false,
            rule_set_revision: None,
        })
        .unwrap()
        .instruction();

    // Fail to validate operation.
    let err = process_failing_validate_ix!(&mut context, validate_ix, vec![], None).await;

    // Check that error is what we expect.  Although the program owner is correct the data length is zero
    // so it fails the rule.
    assert_custom_error!(err, RuleSetError::ProgramOwnedListCheckFailed);
}
