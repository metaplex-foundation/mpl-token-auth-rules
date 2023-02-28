#![cfg(feature = "test-bpf")]

pub mod utils;

use mpl_token_auth_rules::{
    error::RuleSetError,
    instruction::{builders::ValidateBuilder, InstructionBuilder, ValidateArgs},
    payload::{Payload, PayloadType},
    state::{CompareOp, Rule, RuleSetV1},
};
use solana_program::{instruction::InstructionError, pubkey, pubkey::Pubkey};
use solana_program_test::{tokio, ProgramTestContext};
use solana_sdk::{
    instruction::AccountMeta,
    signature::Signer,
    signer::keypair::Keypair,
    system_instruction,
    transaction::{Transaction, TransactionError},
};
use utils::{
    create_associated_token_account, create_mint, program_test, DelegateScenario,
    MetadataDelegateRole, Operation, PayloadKey, TokenDelegateRole, TransferScenario,
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

struct ComposedRules {
    transfer_rule: Rule,
    wallet_to_wallet_rule: Rule,
    delegate_rule: Rule,
    advanced_delegate_rule: Rule,
    namespace_rule: Rule,
}

// Get the four Composed Rules used in this RuleSet.
fn get_composed_rules() -> ComposedRules {
    // --------------------------------
    // Create Primitive Rules
    // --------------------------------
    let nft_amount = Rule::Amount {
        field: PayloadKey::Amount.to_string(),
        amount: 1,
        operator: CompareOp::Eq,
    };

    // Generate some random programs to add to the base lists.
    let random_programs = (0..18).map(|_| Keypair::new().pubkey()).collect::<Vec<_>>();

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

    let source_is_wallet = Rule::IsWallet {
        field: PayloadKey::Source.to_string(),
    };

    let dest_is_wallet = Rule::IsWallet {
        field: PayloadKey::Destination.to_string(),
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

    // (amount is 1 && source is wallet && dest is wallet)
    let wallet_to_wallet_rule = Rule::All {
        rules: vec![nft_amount.clone(), source_is_wallet, dest_is_wallet],
    };

    let delegate_rule = Rule::All {
        rules: vec![nft_amount.clone(), delegate_program_allow_list],
    };

    let advanced_delegate_rule = Rule::All {
        rules: vec![nft_amount, advanced_delegate_program_allow_list],
    };

    let namespace_rule = Rule::Namespace;

    ComposedRules {
        transfer_rule,
        wallet_to_wallet_rule,
        delegate_rule,
        advanced_delegate_rule,
        namespace_rule,
    }
}

fn get_royalty_rule_set(owner: Pubkey) -> RuleSetV1 {
    // Create a RuleSet.
    let mut royalty_rule_set = RuleSetV1::new(RULE_SET_NAME.to_string(), owner);

    // Get transfer and wallet-to-wallet rules.
    let rules = get_composed_rules();

    // --------------------------------
    // Set up transfer operations
    // --------------------------------
    let transfer_operation = Operation::TransferNamespace;
    let transfer_owner_operation = Operation::Transfer {
        scenario: TransferScenario::Holder,
    };

    let transfer_transfer_delegate_operation = Operation::Transfer {
        scenario: TransferScenario::TransferDelegate,
    };

    let transfer_sale_delegate_operation = Operation::Transfer {
        scenario: TransferScenario::SaleDelegate,
    };

    let transfer_migration_delegate_operation = Operation::Transfer {
        scenario: TransferScenario::MigrationDelegate,
    };

    let transfer_wallet_to_wallet_operation = Operation::Transfer {
        scenario: TransferScenario::WalletToWallet,
    };

    royalty_rule_set
        .add(transfer_operation.to_string(), rules.transfer_rule.clone())
        .unwrap();
    royalty_rule_set
        .add(
            transfer_owner_operation.to_string(),
            rules.namespace_rule.clone(),
        )
        .unwrap();
    royalty_rule_set
        .add(
            transfer_transfer_delegate_operation.to_string(),
            rules.namespace_rule.clone(),
        )
        .unwrap();
    royalty_rule_set
        .add(
            transfer_sale_delegate_operation.to_string(),
            rules.namespace_rule.clone(),
        )
        .unwrap();
    royalty_rule_set
        .add(
            transfer_migration_delegate_operation.to_string(),
            rules.namespace_rule.clone(),
        )
        .unwrap();
    royalty_rule_set
        .add(
            transfer_wallet_to_wallet_operation.to_string(),
            rules.wallet_to_wallet_rule,
        )
        .unwrap();

    // --------------------------------
    // Setup metadata delegate operations
    // --------------------------------
    let delegate_operation = Operation::DelegateNamespace;
    let metadata_delegate_authority_operation = Operation::Delegate {
        scenario: DelegateScenario::Metadata(MetadataDelegateRole::Authority),
    };

    let metadata_delegate_collection_operation = Operation::Delegate {
        scenario: DelegateScenario::Metadata(MetadataDelegateRole::Collection),
    };

    let metadata_delegate_use_operation = Operation::Delegate {
        scenario: DelegateScenario::Metadata(MetadataDelegateRole::Use),
    };

    let metadata_delegate_update_operation = Operation::Delegate {
        scenario: DelegateScenario::Metadata(MetadataDelegateRole::Update),
    };

    royalty_rule_set
        .add(delegate_operation.to_string(), rules.delegate_rule.clone())
        .unwrap();

    royalty_rule_set
        .add(
            metadata_delegate_authority_operation.to_string(),
            rules.namespace_rule.clone(),
        )
        .unwrap();
    royalty_rule_set
        .add(
            metadata_delegate_collection_operation.to_string(),
            rules.namespace_rule.clone(),
        )
        .unwrap();
    royalty_rule_set
        .add(
            metadata_delegate_use_operation.to_string(),
            rules.namespace_rule.clone(),
        )
        .unwrap();
    royalty_rule_set
        .add(
            metadata_delegate_update_operation.to_string(),
            rules.namespace_rule.clone(),
        )
        .unwrap();

    // --------------------------------
    // Setup token delegate operations
    // --------------------------------
    let token_delegate_sale_operation = Operation::Delegate {
        scenario: DelegateScenario::Token(TokenDelegateRole::Sale),
    };

    let token_delegate_transfer_operation = Operation::Delegate {
        scenario: DelegateScenario::Token(TokenDelegateRole::Transfer),
    };

    let token_delegate_locked_transfer_operation = Operation::Delegate {
        scenario: DelegateScenario::Token(TokenDelegateRole::LockedTransfer),
    };

    let token_delegate_utility_operation = Operation::Delegate {
        scenario: DelegateScenario::Token(TokenDelegateRole::Utility),
    };

    let token_delegate_staking_operation = Operation::Delegate {
        scenario: DelegateScenario::Token(TokenDelegateRole::Staking),
    };

    royalty_rule_set
        .add(
            token_delegate_sale_operation.to_string(),
            rules.namespace_rule.clone(),
        )
        .unwrap();
    royalty_rule_set
        .add(
            token_delegate_transfer_operation.to_string(),
            rules.namespace_rule.clone(),
        )
        .unwrap();

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
        .add(
            token_delegate_utility_operation.to_string(),
            rules.namespace_rule.clone(),
        )
        .unwrap();

    royalty_rule_set
        .add(
            token_delegate_staking_operation.to_string(),
            rules.namespace_rule,
        )
        .unwrap();

    print!("Royalty Rule Set: {:#?}", royalty_rule_set);

    royalty_rule_set
}

async fn create_royalty_rule_set(context: &mut ProgramTestContext) -> Pubkey {
    let royalty_rule_set = get_royalty_rule_set(context.payer.pubkey());

    // Put the `RuleSet` on chain.
    create_big_rule_set_on_chain!(
        context,
        royalty_rule_set.clone(),
        RULE_SET_NAME.to_string(),
        Some(ADDITIONAL_COMPUTE)
    )
    .await
}

async fn create_incomplete_royalty_rule_set(
    context: &mut ProgramTestContext,
    missing_op: String,
) -> Pubkey {
    let mut royalty_rule_set = get_royalty_rule_set(context.payer.pubkey());
    // Remove a namespaced operation to verify it fails.
    royalty_rule_set.operations.remove(&missing_op);

    // Put the `RuleSet` on chain.
    create_big_rule_set_on_chain!(
        context,
        royalty_rule_set.clone(),
        RULE_SET_NAME.to_string(),
        Some(ADDITIONAL_COMPUTE)
    )
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

    let transfer_wallet_to_wallet_operation = Operation::Transfer {
        scenario: TransferScenario::WalletToWallet,
    };

    // Create a `validate` instruction.
    let validate_ix = ValidateBuilder::new()
        .rule_set_pda(rule_set_addr)
        .mint(mint.pubkey())
        .additional_rule_accounts(vec![
            AccountMeta::new_readonly(source.pubkey(), false),
            AccountMeta::new_readonly(dest.pubkey(), false),
        ])
        .build(ValidateArgs::V1 {
            operation: transfer_wallet_to_wallet_operation.to_string(),
            payload,
            update_rule_state: false,
            rule_set_revision: None,
        })
        .unwrap()
        .instruction();

    // Validate fail operation.
    let err =
        process_failing_validate_ix!(&mut context, validate_ix, vec![], Some(ADDITIONAL_COMPUTE))
            .await;

    // Check that error is what we expect.  The `IsWallet` rule currently returns `NotImplemented`.
    match err {
        solana_program_test::BanksClientError::TransactionError(
            TransactionError::InstructionError(_, InstructionError::Custom(error)),
        ) => {
            assert_eq!(error, RuleSetError::NotImplemented as u32);
        }
        _ => panic!("Unexpected error: {:?}", err),
    }
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
        (
            PayloadKey::Authority.to_string(),
            PayloadType::Pubkey(context.payer.pubkey()),
        ),
    ]);

    let transfer_owner_operation = Operation::Transfer {
        scenario: TransferScenario::Holder,
    };

    // Create a `validate` instruction.
    let validate_ix = ValidateBuilder::new()
        .rule_set_pda(rule_set_addr)
        .mint(mint.pubkey())
        .additional_rule_accounts(vec![
            AccountMeta::new_readonly(source.pubkey(), false),
            AccountMeta::new_readonly(rule_set_addr, false),
            AccountMeta::new_readonly(context.payer.pubkey(), true),
        ])
        .build(ValidateArgs::V1 {
            operation: transfer_owner_operation.to_string(),
            payload,
            update_rule_state: false,
            rule_set_revision: None,
        })
        .unwrap()
        .instruction();

    // Validate operation.
    process_passing_validate_ix!(&mut context, validate_ix, vec![], Some(ADDITIONAL_COMPUTE)).await;
}

#[tokio::test]
async fn wallet_to_prog_owned_missing_namespace() {
    let mut context = program_test().start_with_context().await;
    let rule_set_addr =
        create_incomplete_royalty_rule_set(&mut context, "Transfer:Owner".to_string()).await;

    // Create a Keypair to simulate a token mint address.
    let mint = Keypair::new();

    // Source key is a wallet.
    let source = Keypair::new();

    // Our destination key is going to be an account owned by the mpl-token-auth-rules program.
    // Any one will do so for convenience we just use the RuleSet.

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
        (
            PayloadKey::Authority.to_string(),
            PayloadType::Pubkey(context.payer.pubkey()),
        ),
    ]);

    let transfer_owner_operation = Operation::Transfer {
        scenario: TransferScenario::Holder,
    };

    // Create a `validate` instruction.
    let validate_ix = ValidateBuilder::new()
        .rule_set_pda(rule_set_addr)
        .mint(mint.pubkey())
        .additional_rule_accounts(vec![
            AccountMeta::new_readonly(source.pubkey(), false),
            AccountMeta::new_readonly(rule_set_addr, false),
            AccountMeta::new_readonly(context.payer.pubkey(), true),
        ])
        .build(ValidateArgs::V1 {
            operation: transfer_owner_operation.to_string(),
            payload,
            update_rule_state: false,
            rule_set_revision: None,
        })
        .unwrap()
        .instruction();

    // Fail to validate operation.
    let err =
        process_failing_validate_ix!(&mut context, validate_ix, vec![], Some(ADDITIONAL_COMPUTE))
            .await;

    // Check that error is what we expect.  Program owner was not on the allow list.
    match err {
        solana_program_test::BanksClientError::TransactionError(
            TransactionError::InstructionError(_, InstructionError::Custom(error)),
        ) => {
            assert_eq!(error, RuleSetError::OperationNotFound as u32);
        }
        _ => panic!("Unexpected error: {:?}", err),
    }
}

#[tokio::test]
async fn wallet_to_prog_owned_no_default() {
    let mut context = program_test().start_with_context().await;
    let rule_set_addr =
        create_incomplete_royalty_rule_set(&mut context, "Transfer".to_string()).await;

    // Create a Keypair to simulate a token mint address.
    let mint = Keypair::new();

    // Source key is a wallet.
    let source = Keypair::new();

    // Our destination key is going to be an account owned by the mpl-token-auth-rules program.
    // Any one will do so for convenience we just use the RuleSet.

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
        (
            PayloadKey::Authority.to_string(),
            PayloadType::Pubkey(context.payer.pubkey()),
        ),
    ]);

    let transfer_owner_operation = Operation::Transfer {
        scenario: TransferScenario::Holder,
    };

    // Create a `validate` instruction.
    let validate_ix = ValidateBuilder::new()
        .rule_set_pda(rule_set_addr)
        .mint(mint.pubkey())
        .additional_rule_accounts(vec![
            AccountMeta::new_readonly(source.pubkey(), false),
            AccountMeta::new_readonly(rule_set_addr, false),
            AccountMeta::new_readonly(context.payer.pubkey(), true),
        ])
        .build(ValidateArgs::V1 {
            operation: transfer_owner_operation.to_string(),
            payload,
            update_rule_state: false,
            rule_set_revision: None,
        })
        .unwrap()
        .instruction();

    // Fail to validate operation.
    let err =
        process_failing_validate_ix!(&mut context, validate_ix, vec![], Some(ADDITIONAL_COMPUTE))
            .await;

    // Check that error is what we expect.  Program owner was not on the allow list.
    match err {
        solana_program_test::BanksClientError::TransactionError(
            TransactionError::InstructionError(_, InstructionError::Custom(error)),
        ) => {
            assert_eq!(error, RuleSetError::OperationNotFound as u32);
        }
        _ => panic!("Unexpected error: {:?}", err),
    }
}

#[tokio::test]
async fn prog_owned_to_prog_owned() {
    let mut context = program_test().start_with_context().await;
    let rule_set_addr = create_royalty_rule_set(&mut context).await;

    // Create a Keypair to simulate a token mint address.
    let mint = Keypair::new();

    // Our source and destination keys are going to be accounts owned by the mpl-token-auth-rules
    // program.  Any one will do so for convenience we just use two `RuleSets`.

    // Get first on-chain account.
    let first_on_chain_account = context
        .banks_client
        .get_account(rule_set_addr)
        .await
        .unwrap()
        .unwrap();

    // Account must have nonzero data to count as program-owned.
    assert!(first_on_chain_account.data.iter().any(|&x| x != 0));

    // Verify account ownership.
    assert_eq!(mpl_token_auth_rules::ID, first_on_chain_account.owner);

    // Create destination `RuleSet`.
    let second_rule_set = RuleSetV1::new("second_rule_set".to_string(), context.payer.pubkey());

    let second_rule_set_addr =
        create_rule_set_on_chain!(&mut context, second_rule_set, "second_rule_set".to_string())
            .await;

    // Get second on-chain account.
    let second_on_chain_account = context
        .banks_client
        .get_account(second_rule_set_addr)
        .await
        .unwrap()
        .unwrap();

    // Account must have nonzero data to count as program-owned.
    assert!(second_on_chain_account.data.iter().any(|&x| x != 0));

    // Verify account ownership.
    assert_eq!(mpl_token_auth_rules::ID, second_on_chain_account.owner);

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
        (
            PayloadKey::Authority.to_string(),
            PayloadType::Pubkey(context.payer.pubkey()),
        ),
    ]);

    let transfer_transfer_delegate_operation = Operation::Transfer {
        scenario: TransferScenario::TransferDelegate,
    };

    // Create a `validate` instruction.
    let validate_ix = ValidateBuilder::new()
        .rule_set_pda(rule_set_addr)
        .mint(mint.pubkey())
        .additional_rule_accounts(vec![
            AccountMeta::new_readonly(rule_set_addr, false),
            AccountMeta::new_readonly(second_rule_set_addr, false),
            AccountMeta::new_readonly(context.payer.pubkey(), true),
        ])
        .build(ValidateArgs::V1 {
            operation: transfer_transfer_delegate_operation.to_string(),
            payload,
            update_rule_state: false,
            rule_set_revision: None,
        })
        .unwrap()
        .instruction();

    // Validate operation.
    process_passing_validate_ix!(&mut context, validate_ix, vec![], Some(ADDITIONAL_COMPUTE)).await;
}

#[tokio::test]
async fn prog_owned_to_wallet() {
    let mut context = program_test().start_with_context().await;
    let rule_set_addr = create_royalty_rule_set(&mut context).await;

    // Create a Keypair to simulate a token mint address.
    let mint = Keypair::new();

    // Our source key is going to be an account owned by the mpl-token-auth-rules program.  Any one
    // will do so for convenience we just use the `RuleSet`.

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

    // Destination key is a wallet.
    let dest = Keypair::new();

    let payload = Payload::from([
        (PayloadKey::Amount.to_string(), PayloadType::Number(1)),
        (
            PayloadKey::Source.to_string(),
            PayloadType::Pubkey(rule_set_addr),
        ),
        (
            PayloadKey::Destination.to_string(),
            PayloadType::Pubkey(rule_set_addr),
        ),
        (
            PayloadKey::Authority.to_string(),
            PayloadType::Pubkey(context.payer.pubkey()),
        ),
    ]);

    let transfer_sale_delegate_operation = Operation::Transfer {
        scenario: TransferScenario::SaleDelegate,
    };

    // Create a `validate` instruction.
    let validate_ix = ValidateBuilder::new()
        .rule_set_pda(rule_set_addr)
        .mint(mint.pubkey())
        .additional_rule_accounts(vec![
            AccountMeta::new_readonly(rule_set_addr, false),
            AccountMeta::new_readonly(dest.pubkey(), false),
            AccountMeta::new_readonly(context.payer.pubkey(), true),
        ])
        .build(ValidateArgs::V1 {
            operation: transfer_sale_delegate_operation.to_string(),
            payload,
            update_rule_state: false,
            rule_set_revision: None,
        })
        .unwrap()
        .instruction();

    // Validate operation.
    process_passing_validate_ix!(&mut context, validate_ix, vec![], Some(ADDITIONAL_COMPUTE)).await;
}

#[tokio::test]
async fn wrong_amount_fails() {
    let mut context = program_test().start_with_context().await;
    let rule_set_addr = create_royalty_rule_set(&mut context).await;

    // Create a Keypair to simulate a token mint address.
    let mint = Keypair::new();

    // Our source key is going to be an account owned by the mpl-token-auth-rules program.  Any one
    // will do so for convenience we just use the `RuleSet`.

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

    // Destination key is a wallet.
    let dest = Keypair::new();

    // Store the payload of data to validate against the rule definition, using the WRONG amount.
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
        (
            PayloadKey::Authority.to_string(),
            PayloadType::Pubkey(context.payer.pubkey()),
        ),
    ]);

    let transfer_sale_delegate_operation = Operation::Transfer {
        scenario: TransferScenario::SaleDelegate,
    };

    // Create a `validate` instruction.
    let validate_ix = ValidateBuilder::new()
        .rule_set_pda(rule_set_addr)
        .mint(mint.pubkey())
        .additional_rule_accounts(vec![
            AccountMeta::new_readonly(rule_set_addr, false),
            AccountMeta::new_readonly(dest.pubkey(), false),
            AccountMeta::new_readonly(context.payer.pubkey(), true),
        ])
        .build(ValidateArgs::V1 {
            operation: transfer_sale_delegate_operation.to_string(),
            payload,
            update_rule_state: false,
            rule_set_revision: None,
        })
        .unwrap()
        .instruction();

    // Fail to validate operation.
    let err =
        process_failing_validate_ix!(&mut context, validate_ix, vec![], Some(ADDITIONAL_COMPUTE))
            .await;

    // Check that error is what we expect.  Amount was greater than that allowed in the rule so it
    // failed.
    match err {
        solana_program_test::BanksClientError::TransactionError(
            TransactionError::InstructionError(_, InstructionError::Custom(error)),
        ) => {
            assert_eq!(error, RuleSetError::AmountCheckFailed as u32);
        }
        _ => panic!("Unexpected error: {:?}", err),
    }
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
        (
            PayloadKey::Authority.to_string(),
            PayloadType::Pubkey(context.payer.pubkey()),
        ),
    ]);

    let transfer_owner_operation = Operation::Transfer {
        scenario: TransferScenario::Holder,
    };

    // Create a `validate` instruction.
    let validate_ix = ValidateBuilder::new()
        .rule_set_pda(rule_set_addr)
        .mint(mint.pubkey())
        .additional_rule_accounts(vec![
            AccountMeta::new_readonly(source.pubkey(), false),
            AccountMeta::new_readonly(associated_token_account, false),
            AccountMeta::new_readonly(context.payer.pubkey(), true),
        ])
        .build(ValidateArgs::V1 {
            operation: transfer_owner_operation.to_string(),
            payload,
            update_rule_state: false,
            rule_set_revision: None,
        })
        .unwrap()
        .instruction();

    // Fail to validate operation.
    let err =
        process_failing_validate_ix!(&mut context, validate_ix, vec![], Some(ADDITIONAL_COMPUTE))
            .await;

    // Check that error is what we expect.  Program owner was not on the allow list.
    match err {
        solana_program_test::BanksClientError::TransactionError(
            TransactionError::InstructionError(_, InstructionError::Custom(error)),
        ) => {
            assert_eq!(error, RuleSetError::ProgramOwnedListCheckFailed as u32);
        }
        _ => panic!("Unexpected error: {:?}", err),
    }
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
        (
            PayloadKey::Authority.to_string(),
            PayloadType::Pubkey(context.payer.pubkey()),
        ),
    ]);

    let transfer_owner_operation = Operation::Transfer {
        scenario: TransferScenario::Holder,
    };

    // Create a `validate` instruction.
    let validate_ix = ValidateBuilder::new()
        .rule_set_pda(rule_set_addr)
        .mint(mint.pubkey())
        .additional_rule_accounts(vec![
            AccountMeta::new_readonly(source.pubkey(), false),
            AccountMeta::new_readonly(program_owned_account.pubkey(), false),
            AccountMeta::new_readonly(context.payer.pubkey(), true),
        ])
        .build(ValidateArgs::V1 {
            operation: transfer_owner_operation.to_string(),
            payload,
            update_rule_state: false,
            rule_set_revision: None,
        })
        .unwrap()
        .instruction();

    // Fail to validate operation.
    let err =
        process_failing_validate_ix!(&mut context, validate_ix, vec![], Some(ADDITIONAL_COMPUTE))
            .await;

    // Check that error is what we expect.  Although the program owner is correct the data length is zero
    // so it fails the rule.
    match err {
        solana_program_test::BanksClientError::TransactionError(
            TransactionError::InstructionError(_, InstructionError::Custom(error)),
        ) => {
            assert_eq!(error, RuleSetError::DataIsEmpty as u32);
        }
        _ => panic!("Unexpected error: {:?}", err),
    }
}
