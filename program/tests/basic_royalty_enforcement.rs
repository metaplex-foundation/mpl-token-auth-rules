#![cfg(feature = "test-bpf")]

pub mod utils;

use mpl_token_auth_rules::{
    instruction::{builders::ValidateBuilder, InstructionBuilder, ValidateArgs},
    payload::{LeafInfo, Payload, PayloadType, SeedsVec},
    state::{Rule, RuleSet},
};
use solana_program::{pubkey::Pubkey, system_program};
use solana_program_test::tokio;
use solana_sdk::{instruction::AccountMeta, signature::Signer, signer::keypair::Keypair};
use utils::{
    create_rule_set_on_chain, process_passing_validate_ix, program_test, Operation, PayloadKey,
};

#[tokio::test]
async fn basic_royalty_enforcement() {
    let mut context = program_test().start_with_context().await;

    // --------------------------------
    // Create RuleSet
    // --------------------------------
    static PROGRAM_ALLOW_LIST: [Pubkey; 1] = [mpl_token_auth_rules::ID];

    // OwnerTransfer out rules.
    let source_owned_by_sys_program = Rule::ProgramOwned {
        program: system_program::ID,
        field: PayloadKey::Source.to_string(),
    };

    let dest_program_allow_list = Rule::ProgramOwnedList {
        programs: PROGRAM_ALLOW_LIST.to_vec(),
        field: PayloadKey::Destination.to_string(),
    };

    let dest_pda_match = Rule::PDAMatch {
        program: None,
        pda_field: PayloadKey::Destination.to_string(),
        seeds_field: PayloadKey::DestinationSeeds.to_string(),
    };

    // OwnerTransfer back rules.
    let source_program_allow_list = Rule::ProgramOwnedList {
        programs: PROGRAM_ALLOW_LIST.to_vec(),
        field: PayloadKey::Source.to_string(),
    };

    let source_pda_match = Rule::PDAMatch {
        program: None,
        pda_field: PayloadKey::Source.to_string(),
        seeds_field: PayloadKey::SourceSeeds.to_string(),
    };

    let dest_owned_by_sys_program = Rule::ProgramOwned {
        program: system_program::ID,
        field: PayloadKey::Destination.to_string(),
    };

    // Compose the Owner Transfer rule as follows:
    // (source is a wallet && destination is on allow list && destination is a PDA) ||
    // (source is on allow list && source is a PDA && destination is a wallet)
    let transfer_rule = Rule::Any {
        rules: vec![
            Rule::All {
                rules: vec![
                    source_owned_by_sys_program,
                    dest_program_allow_list,
                    dest_pda_match,
                ],
            },
            Rule::All {
                rules: vec![
                    source_program_allow_list,
                    source_pda_match,
                    dest_owned_by_sys_program,
                ],
            },
        ],
    };

    // Alternative Transfer Rule:
    // (source is a wallet || (source is on allow list && source is a PDA) &&
    // (dest is a wallet || (dest is on allow list && dest is a PDA)
    // let transfer_rule = Rule::All {
    //     rules: vec![
    //         Rule::Any {
    //             rules: vec![
    //                 source_owned_by_sys_program,
    //                 Rule::All {
    //                     rules: vec![source_program_allow_list, source_pda_match],
    //                 },
    //             ],
    //         },
    //         Rule::Any {
    //             rules: vec![
    //                 dest_owned_by_sys_program,
    //                 Rule::All {
    //                     rules: vec![dest_program_allow_list, dest_pda_match],
    //                 },
    //             ],
    //         },
    //     ],
    // };

    // Merkle tree root generated in a different test program.
    let marketplace_tree_root: [u8; 32] = [
        132, 141, 27, 31, 23, 154, 145, 128, 32, 62, 122, 224, 248, 128, 37, 139, 200, 46, 163,
        238, 76, 123, 155, 141, 73, 12, 111, 192, 122, 80, 126, 155,
    ];

    // Rule for Delegate and SaleTransfer: The provided leaf node must be a
    // member of the marketplace Merkle tree.
    let leaf_in_marketplace_tree = Rule::PubkeyTreeMatch {
        root: marketplace_tree_root,
        field: PayloadKey::Destination.to_string(),
    };

    // Create Basic Royalty Enforcement RuleSet.
    let mut basic_royalty_enforcement_rule_set = RuleSet::new(
        "basic_royalty_enforcement".to_string(),
        context.payer.pubkey(),
    );
    basic_royalty_enforcement_rule_set
        .add(Operation::OwnerTransfer.to_string(), transfer_rule)
        .unwrap();
    basic_royalty_enforcement_rule_set
        .add(
            Operation::Delegate.to_string(),
            leaf_in_marketplace_tree.clone(),
        )
        .unwrap();
    basic_royalty_enforcement_rule_set
        .add(
            Operation::SaleTransfer.to_string(),
            leaf_in_marketplace_tree,
        )
        .unwrap();

    println!(
        "{}",
        serde_json::to_string_pretty(&basic_royalty_enforcement_rule_set,).unwrap()
    );

    // Put the RuleSet on chain.
    let rule_set_addr = create_rule_set_on_chain(
        &mut context,
        basic_royalty_enforcement_rule_set,
        "basic_royalty_enforcement".to_string(),
    )
    .await;

    // --------------------------------
    // Validate Transfer to a PDA.
    // --------------------------------
    // Create a Keypair to simulate an owner's wallet.
    let wallet = Keypair::new().pubkey();

    // Create a Keypair to simulate a token mint address.
    let mint = Keypair::new().pubkey();

    // Our derived key is going to be an account owned by the
    // mpl-token-auth-rules program. Any one will do so for convenience
    // we just use the RuleSet.  These are the RuleSet seeds.
    let seeds = vec![
        mpl_token_auth_rules::pda::PREFIX.as_bytes().to_vec(),
        context.payer.pubkey().as_ref().to_vec(),
        "basic_royalty_enforcement".as_bytes().to_vec(),
    ];

    // Store the payload of data to validate against the rule definition.  In this case the
    // `Destination` will be used to look up the `AccountInfo` and see and see who the owner
    // is, and the `DestinationSeeds` provide the seeds for the PDA derivation.
    let payload = Payload::from([
        (PayloadKey::Source.to_string(), PayloadType::Pubkey(wallet)),
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
        .mint(mint)
        .additional_rule_accounts(vec![
            AccountMeta::new_readonly(wallet, false),
            AccountMeta::new_readonly(rule_set_addr, false),
        ])
        .build(ValidateArgs::V1 {
            operation: Operation::OwnerTransfer.to_string(),
            payload,
            update_rule_state: false,
        })
        .unwrap()
        .instruction();

    // Validate Transfer operation.
    process_passing_validate_ix(&mut context, validate_ix, vec![]).await;

    // --------------------------------
    // Validate Transfer to a wallet.
    // --------------------------------
    // Store the payload of data to validate against the rule definition.
    // In this case the Target will be used to look up the `AccountInfo`
    // and see who the owner is.
    let payload = Payload::from([
        (
            PayloadKey::Source.to_string(),
            PayloadType::Pubkey(rule_set_addr),
        ),
        (
            PayloadKey::SourceSeeds.to_string(),
            PayloadType::Seeds(SeedsVec::new(seeds)),
        ),
        (
            PayloadKey::Destination.to_string(),
            PayloadType::Pubkey(wallet),
        ),
    ]);

    let validate_ix = ValidateBuilder::new()
        .rule_set_pda(rule_set_addr)
        .mint(mint)
        .additional_rule_accounts(vec![
            AccountMeta::new_readonly(rule_set_addr, false),
            AccountMeta::new_readonly(wallet, false),
        ])
        .build(ValidateArgs::V1 {
            operation: Operation::OwnerTransfer.to_string(),
            payload,
            update_rule_state: false,
        })
        .unwrap()
        .instruction();

    // Validate Transfer operation.
    process_passing_validate_ix(&mut context, validate_ix, vec![]).await;

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
        .mint(mint)
        .additional_rule_accounts(vec![])
        .build(ValidateArgs::V1 {
            operation: Operation::Delegate.to_string(),
            payload: payload.clone(),
            update_rule_state: false,
        })
        .unwrap()
        .instruction();

    // Validate Delegate operation.
    process_passing_validate_ix(&mut context, validate_ix, vec![]).await;

    // --------------------------------
    // Validate SaleTransfer operation
    // --------------------------------
    // Create a `validate` instruction for a `SaleTransfer` operation.
    let validate_ix = ValidateBuilder::new()
        .rule_set_pda(rule_set_addr)
        .mint(mint)
        .additional_rule_accounts(vec![])
        .build(ValidateArgs::V1 {
            operation: Operation::SaleTransfer.to_string(),
            payload,
            update_rule_state: false,
        })
        .unwrap()
        .instruction();

    // Validate SaleTransfer operation.
    process_passing_validate_ix(&mut context, validate_ix, vec![]).await;
}
