#![cfg(feature = "test-bpf")]

pub mod utils;

use mpl_token_auth_rules::{
    error::RuleSetError,
    instruction::{builders::ValidateBuilder, InstructionBuilder, ValidateArgs},
    payload::{LeafInfo, Payload, PayloadType, SeedsVec},
    state::{CompareOp, Rule, RuleSet},
};
use solana_program::{pubkey::Pubkey, system_program};
use solana_program_test::tokio;
use solana_sdk::{
    instruction::AccountMeta, signature::Signer, signer::keypair::Keypair, system_instruction,
    transaction::Transaction,
};
use utils::{create_associated_token_account, create_mint, program_test, Operation, PayloadKey};

static PROGRAM_ALLOW_LIST: [Pubkey; 1] = [mpl_token_auth_rules::ID];

macro_rules! get_primitive_rules {
    (
        $nft_amount:ident,
        $source_owned_by_sys_program:ident,
        $source_program_allow_list:ident,
        $source_pda_match:ident,
        $dest_owned_by_sys_program:ident,
        $dest_program_allow_list:ident,
        $dest_pda_match:ident
    ) => {
        let $nft_amount = Rule::Amount {
            field: PayloadKey::Amount.to_string(),
            amount: 1,
            operator: CompareOp::Eq,
        };

        let $source_owned_by_sys_program = Rule::ProgramOwned {
            program: system_program::ID,
            field: PayloadKey::Source.to_string(),
        };

        let $source_program_allow_list = Rule::ProgramOwnedList {
            programs: PROGRAM_ALLOW_LIST.to_vec(),
            field: PayloadKey::Source.to_string(),
        };

        let $source_pda_match = Rule::PDAMatch {
            program: None,
            pda_field: PayloadKey::Source.to_string(),
            seeds_field: PayloadKey::SourceSeeds.to_string(),
        };

        let $dest_owned_by_sys_program = Rule::ProgramOwned {
            program: system_program::ID,
            field: PayloadKey::Destination.to_string(),
        };

        let $dest_program_allow_list = Rule::ProgramOwnedList {
            programs: PROGRAM_ALLOW_LIST.to_vec(),
            field: PayloadKey::Destination.to_string(),
        };

        let $dest_pda_match = Rule::PDAMatch {
            program: None,
            pda_field: PayloadKey::Destination.to_string(),
            seeds_field: PayloadKey::DestinationSeeds.to_string(),
        };
    };
}

#[tokio::test]
async fn sys_prog_owned_or_owned_pda_to_sys_prog_owned_or_owned_pda() {
    let mut context = program_test().start_with_context().await;

    get_primitive_rules!(
        nft_amount,
        source_owned_by_sys_program,
        source_program_allow_list,
        source_pda_match,
        dest_owned_by_sys_program,
        dest_program_allow_list,
        dest_pda_match
    );

    // --------------------------------
    // Create RuleSet
    // --------------------------------
    // Compose the Owner Transfer rule as follows:
    // (source is owned by system program || (source is on allow list && source is a PDA) &&
    // (dest is owned by system program || (dest is on allow list && dest is a PDA)
    let transfer_rule = Rule::All {
        rules: vec![
            nft_amount,
            Rule::Any {
                rules: vec![
                    source_owned_by_sys_program,
                    Rule::All {
                        rules: vec![source_program_allow_list, source_pda_match],
                    },
                ],
            },
            Rule::Any {
                rules: vec![
                    dest_owned_by_sys_program,
                    Rule::All {
                        rules: vec![dest_program_allow_list, dest_pda_match],
                    },
                ],
            },
        ],
    };

    // Create RuleSet.
    let mut rule_set = RuleSet::new(
        "basic_royalty_enforcement".to_string(),
        context.payer.pubkey(),
    );
    rule_set
        .add(Operation::OwnerTransfer.to_string(), transfer_rule)
        .unwrap();

    println!("{}", serde_json::to_string_pretty(&rule_set,).unwrap());

    // Put the RuleSet on chain.
    let rule_set_addr = create_rule_set_on_chain!(
        &mut context,
        rule_set.clone(),
        "basic_royalty_enforcement".to_string()
    )
    .await;

    // --------------------------------
    // Validate sys prog owned to sys prog owned
    // --------------------------------
    let source = Keypair::new();
    let dest = Keypair::new();

    // Create a Keypair to simulate a token mint address.
    let mint = Keypair::new();

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

    let validate_ix = ValidateBuilder::new()
        .rule_set_pda(rule_set_addr)
        .mint(mint.pubkey())
        .additional_rule_accounts(vec![
            AccountMeta::new_readonly(source.pubkey(), false),
            AccountMeta::new_readonly(dest.pubkey(), false),
        ])
        .build(ValidateArgs::V1 {
            operation: Operation::OwnerTransfer.to_string(),
            payload,
            update_rule_state: false,
        })
        .unwrap()
        .instruction();

    // Validate OwnerTransfer operation.
    process_passing_validate_ix!(&mut context, validate_ix, vec![]).await;

    // --------------------------------
    // Validate sys prog owned to prog owned PDA
    // --------------------------------
    // Our derived key is going to be an account owned by the
    // mpl-token-auth-rules program. Any one will do so for convenience
    // we just use the RuleSet.  These are the RuleSet seeds.
    let seeds = vec![
        mpl_token_auth_rules::pda::PREFIX.as_bytes().to_vec(),
        context.payer.pubkey().as_ref().to_vec(),
        "basic_royalty_enforcement".as_bytes().to_vec(),
    ];

    // Store the payload of data to validate against the rule definition.
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
            PayloadKey::DestinationSeeds.to_string(),
            PayloadType::Seeds(SeedsVec::new(seeds.clone())),
        ),
    ]);

    let validate_ix = ValidateBuilder::new()
        .rule_set_pda(rule_set_addr)
        .mint(mint.pubkey())
        .additional_rule_accounts(vec![
            AccountMeta::new_readonly(source.pubkey(), false),
            AccountMeta::new_readonly(rule_set_addr, false),
        ])
        .build(ValidateArgs::V1 {
            operation: Operation::OwnerTransfer.to_string(),
            payload,
            update_rule_state: false,
        })
        .unwrap()
        .instruction();

    // Validate OwnerTransfer operation.
    process_passing_validate_ix!(&mut context, validate_ix, vec![]).await;

    // --------------------------------
    // Validate prog owned PDA to prog owned PDA
    // --------------------------------
    // Create a second RuleSet on chain for the sole purpose of having
    // another PDA that is owned by the mpl-token-auth-rules program.
    let second_rule_set = RuleSet::new("second_rule_set".to_string(), context.payer.pubkey());

    let second_rule_set_addr =
        create_rule_set_on_chain!(&mut context, second_rule_set, "second_rule_set".to_string())
            .await;

    let second_rule_set_seeds = vec![
        mpl_token_auth_rules::pda::PREFIX.as_bytes().to_vec(),
        context.payer.pubkey().as_ref().to_vec(),
        "second_rule_set".as_bytes().to_vec(),
    ];

    // Store the payload of data to validate against the rule definition.
    let payload = Payload::from([
        (PayloadKey::Amount.to_string(), PayloadType::Number(1)),
        (
            PayloadKey::Source.to_string(),
            PayloadType::Pubkey(rule_set_addr),
        ),
        (
            PayloadKey::SourceSeeds.to_string(),
            PayloadType::Seeds(SeedsVec::new(seeds.clone())),
        ),
        (
            PayloadKey::Destination.to_string(),
            PayloadType::Pubkey(second_rule_set_addr),
        ),
        (
            PayloadKey::DestinationSeeds.to_string(),
            PayloadType::Seeds(SeedsVec::new(second_rule_set_seeds)),
        ),
    ]);

    let validate_ix = ValidateBuilder::new()
        .rule_set_pda(rule_set_addr)
        .mint(mint.pubkey())
        .additional_rule_accounts(vec![
            AccountMeta::new_readonly(rule_set_addr, false),
            AccountMeta::new_readonly(second_rule_set_addr, false),
        ])
        .build(ValidateArgs::V1 {
            operation: Operation::OwnerTransfer.to_string(),
            payload,
            update_rule_state: false,
        })
        .unwrap()
        .instruction();

    // Validate OwnerTransfer operation.
    process_passing_validate_ix!(&mut context, validate_ix, vec![]).await;

    // --------------------------------
    // Validate prog owned PDA to sys prog owned
    // --------------------------------
    // Store the payload of data to validate against the rule definition.
    let payload = Payload::from([
        (PayloadKey::Amount.to_string(), PayloadType::Number(1)),
        (
            PayloadKey::Source.to_string(),
            PayloadType::Pubkey(rule_set_addr),
        ),
        (
            PayloadKey::SourceSeeds.to_string(),
            PayloadType::Seeds(SeedsVec::new(seeds.clone())),
        ),
        (
            PayloadKey::Destination.to_string(),
            PayloadType::Pubkey(dest.pubkey()),
        ),
    ]);

    let validate_ix = ValidateBuilder::new()
        .rule_set_pda(rule_set_addr)
        .mint(mint.pubkey())
        .additional_rule_accounts(vec![
            AccountMeta::new_readonly(rule_set_addr, false),
            AccountMeta::new_readonly(dest.pubkey(), false),
        ])
        .build(ValidateArgs::V1 {
            operation: Operation::OwnerTransfer.to_string(),
            payload,
            update_rule_state: false,
        })
        .unwrap()
        .instruction();

    // Validate OwnerTransfer operation.
    process_passing_validate_ix!(&mut context, validate_ix, vec![]).await;

    // --------------------------------
    // Validate fail wrong amount
    // --------------------------------
    // Create a Keypair to simulate a token mint address.
    let mint = Keypair::new();

    // Store the payload of data to validate against the rule definition.
    let payload = Payload::from([
        (PayloadKey::Amount.to_string(), PayloadType::Number(2)),
        (
            PayloadKey::Source.to_string(),
            PayloadType::Pubkey(source.pubkey()),
        ),
        (
            PayloadKey::Destination.to_string(),
            PayloadType::Pubkey(dest.pubkey()),
        ),
    ]);

    let validate_ix = ValidateBuilder::new()
        .rule_set_pda(rule_set_addr)
        .mint(mint.pubkey())
        .additional_rule_accounts(vec![
            AccountMeta::new_readonly(source.pubkey(), false),
            AccountMeta::new_readonly(dest.pubkey(), false),
        ])
        .build(ValidateArgs::V1 {
            operation: Operation::OwnerTransfer.to_string(),
            payload,
            update_rule_state: false,
        })
        .unwrap()
        .instruction();

    // Fail to validate OwnerTransfer operation.
    let err = process_failing_validate_ix!(&mut context, validate_ix, vec![]).await;

    // Check that error is what we expect.
    assert_rule_set_error!(err, RuleSetError::AmountCheckFailed);

    // --------------------------------
    // Validate fail not sys prog owned, valid PDA, but not prog owned
    // --------------------------------
    // Create an associated token account for the sole purpose of having
    // a valid PDA that is owned by a different program than what is in the Rule.
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

    let associated_token_account_seeds = vec![
        source.pubkey().to_bytes().to_vec(),
        spl_token::ID.to_bytes().to_vec(),
        mint.to_bytes().to_vec(),
    ];

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
            PayloadKey::DestinationSeeds.to_string(),
            PayloadType::Seeds(SeedsVec::new(associated_token_account_seeds)),
        ),
    ]);

    let validate_ix = ValidateBuilder::new()
        .rule_set_pda(rule_set_addr)
        .mint(mint.pubkey())
        .additional_rule_accounts(vec![
            AccountMeta::new_readonly(source.pubkey(), false),
            AccountMeta::new_readonly(associated_token_account, false),
        ])
        .build(ValidateArgs::V1 {
            operation: Operation::OwnerTransfer.to_string(),
            payload,
            update_rule_state: false,
        })
        .unwrap()
        .instruction();

    // Fail to validate OwnerTransfer operation.
    let err = process_failing_validate_ix!(&mut context, validate_ix, vec![]).await;

    // Check that error is what we expect.  It should fail the ProgramOwnedList Rule since the
    // owner is not in the Rule.
    assert_rule_set_error!(err, RuleSetError::ProgramOwnedListCheckFailed);

    // --------------------------------
    // Validate fail prog owned but not a PDA
    // --------------------------------
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
        (
            PayloadKey::DestinationSeeds.to_string(),
            PayloadType::Seeds(SeedsVec::new(seeds)),
        ),
    ]);

    let validate_ix = ValidateBuilder::new()
        .rule_set_pda(rule_set_addr)
        .mint(mint.pubkey())
        .additional_rule_accounts(vec![
            AccountMeta::new_readonly(source.pubkey(), false),
            AccountMeta::new_readonly(program_owned_account.pubkey(), false),
        ])
        .build(ValidateArgs::V1 {
            operation: Operation::OwnerTransfer.to_string(),
            payload,
            update_rule_state: false,
        })
        .unwrap()
        .instruction();

    // Fail to validate Transfer operation.
    let err = process_failing_validate_ix!(&mut context, validate_ix, vec![]).await;

    // Check that error is what we expect.  It should fail the PDAMatch Rule after passing
    // the ProgramOwnedList Rule, since the owner was correct but it is not a valid PDA.
    assert_rule_set_error!(err, RuleSetError::PDAMatchCheckFailed);
}

#[tokio::test]
async fn multiple_operations() {
    let mut context = program_test().start_with_context().await;

    get_primitive_rules!(
        _nft_amount,
        source_owned_by_sys_program,
        _source_program_allow_list,
        _source_pda_match,
        _dest_owned_by_sys_program,
        dest_program_allow_list,
        dest_pda_match
    );

    // --------------------------------
    // Create RuleSet
    // --------------------------------
    // Compose the Owner Transfer rule as follows:
    // (source is a owned by system program && dest is on allow list && dest is a PDA)
    let transfer_rule = Rule::All {
        rules: vec![
            source_owned_by_sys_program,
            dest_program_allow_list,
            dest_pda_match,
        ],
    };

    // Merkle tree root generated in a different test program.
    let marketplace_tree_root: [u8; 32] = [
        132, 141, 27, 31, 23, 154, 145, 128, 32, 62, 122, 224, 248, 128, 37, 139, 200, 46, 163,
        238, 76, 123, 155, 141, 73, 12, 111, 192, 122, 80, 126, 155,
    ];

    // The provided leaf node must be a member of the marketplace Merkle tree.
    let leaf_in_marketplace_tree = Rule::PubkeyTreeMatch {
        root: marketplace_tree_root,
        field: PayloadKey::Destination.to_string(),
    };

    // Create RuleSet.
    let mut rule_set = RuleSet::new(
        "basic_royalty_enforcement".to_string(),
        context.payer.pubkey(),
    );
    rule_set
        .add(Operation::OwnerTransfer.to_string(), transfer_rule)
        .unwrap();

    rule_set
        .add(
            Operation::Delegate.to_string(),
            leaf_in_marketplace_tree.clone(),
        )
        .unwrap();
    rule_set
        .add(
            Operation::SaleTransfer.to_string(),
            leaf_in_marketplace_tree,
        )
        .unwrap();

    println!("{}", serde_json::to_string_pretty(&rule_set,).unwrap());

    // Put the RuleSet on chain.
    let rule_set_addr = create_rule_set_on_chain!(
        &mut context,
        rule_set,
        "basic_royalty_enforcement".to_string()
    )
    .await;

    // --------------------------------
    // Validate sys prog owned to PDA
    // --------------------------------
    let source = Keypair::new();

    // Create a Keypair to simulate a token mint address.
    let mint = Keypair::new();

    // Our derived key is going to be an account owned by the
    // mpl-token-auth-rules program. Any one will do so for convenience
    // we just use the RuleSet.  These are the RuleSet seeds.
    let seeds = vec![
        mpl_token_auth_rules::pda::PREFIX.as_bytes().to_vec(),
        context.payer.pubkey().as_ref().to_vec(),
        "basic_royalty_enforcement".as_bytes().to_vec(),
    ];

    // Store the payload of data to validate against the rule definition.
    let payload = Payload::from([
        (
            PayloadKey::Source.to_string(),
            PayloadType::Pubkey(source.pubkey()),
        ),
        (
            PayloadKey::Destination.to_string(),
            PayloadType::Pubkey(rule_set_addr),
        ),
        (
            PayloadKey::DestinationSeeds.to_string(),
            PayloadType::Seeds(SeedsVec::new(seeds.clone())),
        ),
    ]);

    let validate_ix = ValidateBuilder::new()
        .rule_set_pda(rule_set_addr)
        .mint(mint.pubkey())
        .additional_rule_accounts(vec![
            AccountMeta::new_readonly(source.pubkey(), false),
            AccountMeta::new_readonly(rule_set_addr, false),
        ])
        .build(ValidateArgs::V1 {
            operation: Operation::OwnerTransfer.to_string(),
            payload,
            update_rule_state: false,
        })
        .unwrap()
        .instruction();

    // Validate OwnerTransfer operation.
    process_passing_validate_ix!(&mut context, validate_ix, vec![]).await;

    // --------------------------------
    // Validate fail sys prog owned to sys prog owned
    // --------------------------------
    let dest = Keypair::new();
    let payload = Payload::from([
        (
            PayloadKey::Source.to_string(),
            PayloadType::Pubkey(source.pubkey()),
        ),
        (
            PayloadKey::Destination.to_string(),
            PayloadType::Pubkey(dest.pubkey()),
        ),
    ]);

    let validate_ix = ValidateBuilder::new()
        .rule_set_pda(rule_set_addr)
        .mint(mint.pubkey())
        .additional_rule_accounts(vec![
            AccountMeta::new_readonly(source.pubkey(), false),
            AccountMeta::new_readonly(dest.pubkey(), false),
        ])
        .build(ValidateArgs::V1 {
            operation: Operation::OwnerTransfer.to_string(),
            payload,
            update_rule_state: false,
        })
        .unwrap()
        .instruction();

    // Fail to validate Transfer operation.
    let err = process_failing_validate_ix!(&mut context, validate_ix, vec![]).await;

    // Check that error is what we expect.  The destination is owned by the System Program
    // so in this case it doesn't match the ProgramOwnedList Rule.
    assert_rule_set_error!(err, RuleSetError::ProgramOwnedListCheckFailed);

    // --------------------------------
    // Validate Delegate operation
    // --------------------------------
    // Merkle tree leaf node.
    let leaf: [u8; 32] = [
        2, 157, 245, 156, 21, 37, 147, 96, 42, 190, 206, 14, 24, 1, 106, 49, 167, 236, 38, 73, 98,
        53, 60, 9, 154, 31, 240, 126, 210, 197, 76, 7,
    ];

    // Merkle tree proof generated in a different test program.
    let proof: Vec<[u8; 32]> = vec![
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

    let leaf_info = LeafInfo::new(leaf, proof);

    // Store the payload of data to validate against the rule definition.
    // In this case it is a leaf node and its associated Merkle proof.
    let payload = Payload::from([(
        PayloadKey::Destination.to_string(),
        PayloadType::MerkleProof(leaf_info),
    )]);

    // Create a `validate` instruction for a `Delegate` operation.
    let validate_ix = ValidateBuilder::new()
        .rule_set_pda(rule_set_addr)
        .mint(mint.pubkey())
        .additional_rule_accounts(vec![])
        .build(ValidateArgs::V1 {
            operation: Operation::Delegate.to_string(),
            payload: payload.clone(),
            update_rule_state: false,
        })
        .unwrap()
        .instruction();

    // Validate Delegate operation.
    process_passing_validate_ix!(&mut context, validate_ix, vec![]).await;

    // --------------------------------
    // Validate SaleTransfer operation
    // --------------------------------
    // Create a `validate` instruction for a `SaleTransfer` operation.
    let validate_ix = ValidateBuilder::new()
        .rule_set_pda(rule_set_addr)
        .mint(mint.pubkey())
        .additional_rule_accounts(vec![])
        .build(ValidateArgs::V1 {
            operation: Operation::SaleTransfer.to_string(),
            payload,
            update_rule_state: false,
        })
        .unwrap()
        .instruction();

    // Validate SaleTransfer operation.
    process_passing_validate_ix!(&mut context, validate_ix, vec![]).await;
}
