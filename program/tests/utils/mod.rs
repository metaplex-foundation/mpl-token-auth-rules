use mpl_token_auth_rules::{
    instruction::{
        builders::{CreateOrUpdateBuilder, PuffRuleSetBuilder, WriteToBufferBuilder},
        CreateOrUpdateArgs, InstructionBuilder, PuffRuleSetArgs, WriteToBufferArgs,
    },
    payload::ProofInfo,
    state::RuleSetV1,
};
use rmp_serde::Serializer;
use serde::Serialize;
use solana_program::{instruction::Instruction, pubkey::Pubkey};
use solana_program_test::{BanksClientError, ProgramTest, ProgramTestContext};
use solana_sdk::{
    compute_budget::ComputeBudgetInstruction, program_pack::Pack, signature::Signer,
    signer::keypair::Keypair, system_instruction, transaction::Transaction,
};
use std::fmt::Display;

// --------------------------------
// RuleSet operations and scenarios
// from token-metadata
// --------------------------------
// Type from token-metadata.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TransferScenario {
    Holder,
    TransferDelegate,
    SaleDelegate,
    MigrationDelegate,
    WalletToWallet,
}

impl Display for TransferScenario {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Holder => write!(f, "Owner"),
            Self::TransferDelegate => write!(f, "TransferDelegate"),
            Self::SaleDelegate => write!(f, "SaleDelegate"),
            Self::MigrationDelegate => write!(f, "MigrationDelegate"),
            Self::WalletToWallet => write!(f, "WalletToWallet"),
        }
    }
}

// Type from token-metadata.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UpdateScenario {
    MetadataAuth,
    Delegate,
    Proxy,
}

impl Display for UpdateScenario {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UpdateScenario::MetadataAuth => write!(f, "MetadataAuth"),
            UpdateScenario::Delegate => write!(f, "Delegate"),
            UpdateScenario::Proxy => write!(f, "Proxy"),
        }
    }
}

// Type from token-metadata.
#[repr(C)]
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum MetadataDelegateRole {
    Authority,
    Collection,
    Use,
    Update,
}

#[repr(C)]
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum TokenDelegateRole {
    Sale,
    Transfer,
    Utility,
    Staking,
    Standard,
    LockedTransfer,
    Migration = 255,
}

// Type from token-metadata.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DelegateScenario {
    Metadata(MetadataDelegateRole),
    Token(TokenDelegateRole),
}

impl Display for DelegateScenario {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::Metadata(role) => match role {
                MetadataDelegateRole::Authority => "Authority".to_string(),
                MetadataDelegateRole::Collection => "Collection".to_string(),
                MetadataDelegateRole::Use => "Use".to_string(),
                MetadataDelegateRole::Update => "Update".to_string(),
            },
            Self::Token(role) => match role {
                TokenDelegateRole::Sale => "Sale".to_string(),
                TokenDelegateRole::Transfer => "Transfer".to_string(),
                TokenDelegateRole::LockedTransfer => "LockedTransfer".to_string(),
                TokenDelegateRole::Utility => "Utility".to_string(),
                TokenDelegateRole::Staking => "Staking".to_string(),
                _ => panic!("Invalid delegate role"),
            },
        };

        write!(f, "{message}")
    }
}

// Type from token-metadata.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Operation {
    Transfer { scenario: TransferScenario },
    TransferNamespace,
    Update { scenario: UpdateScenario },
    UpdateNamespace,
    Delegate { scenario: DelegateScenario },
    DelegateNamespace,
}

impl ToString for Operation {
    fn to_string(&self) -> String {
        match self {
            Self::Transfer { scenario } => format!("Transfer:{}", scenario),
            Self::TransferNamespace => "Transfer".to_string(),
            Self::Update { scenario } => format!("Update:{}", scenario),
            Self::UpdateNamespace => "Update".to_string(),
            Self::Delegate { scenario } => format!("Delegate:{}", scenario),
            Self::DelegateNamespace => "Delegate".to_string(),
        }
    }
}

// Payload key type from token-metadata.
#[repr(C)]
#[derive(PartialEq, Eq, Debug, Clone)]
pub enum PayloadKey {
    /// The amount being transferred.
    Amount,
    /// The authority of an operation, e.g. the delegate of token.
    Authority,
    /// Seeds for a PDA authority of the operation, e.g. when the authority is a PDA.
    AuthoritySeeds,
    /// Merkle proof for the source of the operation, e.g. when the authority is a member
    /// of a Merkle tree.
    AuthorityProof,
    /// Delegate for an operation.
    Delegate,
    /// Seeds for a PDA delegate of the operation.
    DelegateSeeds,
    /// The destination of the operation, e.g. the recipient of a transfer.
    Destination,
    /// Seeds for a PDA destination of the operation, e.g. when the recipient is a PDA.
    DestinationSeeds,
    /// A token holder.
    Holder,
    /// The source of the operation, e.g. the owner initiating a transfer.
    Source,
    /// Seeds for a PDA source of the operation, e.g. when the source is a PDA.
    SourceSeeds,
    /// Merkle proof for the source of the operation, e.g. when the source is a member
    /// of a Merkle tree.
    SourceProof,
    /// Merkle proof for the destination of the operation, e.g. when the distination
    /// is a member of a Merkle tree.
    DestinationProof,
}

impl ToString for PayloadKey {
    fn to_string(&self) -> String {
        match self {
            PayloadKey::Amount => "Amount",
            PayloadKey::Authority => "Authority",
            PayloadKey::AuthoritySeeds => "AuthoritySeeds",
            PayloadKey::AuthorityProof => "AuthorityProof",
            PayloadKey::Delegate => "Delegate",
            PayloadKey::DelegateSeeds => "DelegateSeeds",
            PayloadKey::SourceProof => "SourceProof",
            PayloadKey::Destination => "Destination",
            PayloadKey::DestinationSeeds => "DestinationSeeds",
            PayloadKey::DestinationProof => "DestinationProof",
            PayloadKey::Holder => "Holder",
            PayloadKey::Source => "Source",
            PayloadKey::SourceSeeds => "SourceSeeds",
        }
        .to_string()
    }
}

pub fn program_test() -> ProgramTest {
    ProgramTest::new("mpl_token_auth_rules", mpl_token_auth_rules::id(), None)
}

#[macro_export]
macro_rules! create_rule_set_on_chain {
    ($context:expr, $rule_set:expr, $rule_set_name:expr) => {
        $crate::utils::create_rule_set_on_chain_with_loc(
            $context,
            $rule_set,
            $rule_set_name,
            file!(),
            line!(),
            column!(),
        )
    };
}

pub async fn create_rule_set_on_chain_with_loc(
    context: &mut ProgramTestContext,
    rule_set: RuleSetV1,
    rule_set_name: String,
    file: &str,
    line: u32,
    column: u32,
) -> Pubkey {
    // Find RuleSet PDA.
    let (rule_set_addr, _rule_set_bump) =
        mpl_token_auth_rules::pda::find_rule_set_address(context.payer.pubkey(), rule_set_name);

    // Serialize the RuleSet using RMP serde.
    let mut serialized_rule_set = Vec::new();
    rule_set
        .serialize(&mut Serializer::new(&mut serialized_rule_set))
        .unwrap();

    // Create a `create_or_update` instruction.
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

    assert!(
        create_tx.message.serialize().len() <= 1232,
        "Transaction exceeds packet limit of 1232"
    );

    // Process the transaction.
    context
        .banks_client
        .process_transaction(create_tx)
        .await
        .unwrap_or_else(|err| {
            panic!(
                "Creation error {:?}, create_rule_set_on_chain called at {}:{}:{}",
                err, file, line, column
            )
        });

    rule_set_addr
}

#[macro_export]
macro_rules! create_big_rule_set_on_chain {
    ($context:expr, $rule_set:expr, $rule_set_name:expr, $compute_budget:expr) => {
        $crate::utils::create_big_rule_set_on_chain_with_loc(
            $context,
            $rule_set,
            $rule_set_name,
            $compute_budget,
            file!(),
            line!(),
            column!(),
        )
    };
}

pub async fn create_big_rule_set_on_chain_with_loc(
    context: &mut ProgramTestContext,
    rule_set: RuleSetV1,
    rule_set_name: String,
    compute_budget: Option<u32>,
    file: &str,
    line: u32,
    column: u32,
) -> Pubkey {
    // Find RuleSet PDA.
    let (rule_set_addr, _rule_set_bump) = mpl_token_auth_rules::pda::find_rule_set_address(
        context.payer.pubkey(),
        rule_set_name.clone(),
    );

    let (buffer_pda, _buffer_bump) =
        mpl_token_auth_rules::pda::find_buffer_address(context.payer.pubkey());

    // Serialize the RuleSet using RMP serde.
    let mut serialized_rule_set = Vec::new();
    rule_set
        .serialize(&mut Serializer::new(&mut serialized_rule_set))
        .unwrap();

    let mut overwrite = true;
    for serialized_rule_set_chunk in serialized_rule_set.chunks(750) {
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

        assert!(
            write_to_buffer_tx.message.serialize().len() <= 1232,
            "Transaction exceeds packet limit of 1232"
        );

        // Process the transaction.
        context
            .banks_client
            .process_transaction(write_to_buffer_tx)
            .await
            .unwrap_or_else(|err| {
                panic!(
                    "Creation error {:?}, create_big_rule_set_on_chain called at {}:{}:{}",
                    err, file, line, column
                )
            });

        if overwrite {
            overwrite = false;
        }
    }
    let data = context
        .banks_client
        .get_account(buffer_pda)
        .await
        .unwrap()
        .unwrap()
        .data;

    assert!(
        cmp_slice(&data, &serialized_rule_set),
        "The buffer doesn't match the serialized rule set.",
    );

    let puff_ix = PuffRuleSetBuilder::new()
        .payer(context.payer.pubkey())
        .rule_set_pda(rule_set_addr)
        .build(PuffRuleSetArgs::V1 {
            rule_set_name: rule_set_name.to_string(),
        })
        .unwrap()
        .instruction();

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

    // Use user-provided compute budget if one was provided.
    let instructions = match compute_budget {
        Some(units) => {
            let compute_budget_ix = ComputeBudgetInstruction::set_compute_unit_limit(units);
            vec![compute_budget_ix, puff_ix, create_ix]
        }
        None => vec![puff_ix, create_ix],
    };

    // Add it to a transaction.
    let create_tx = Transaction::new_signed_with_payer(
        &instructions,
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    assert!(
        create_tx.message.serialize().len() <= 1232,
        "Transaction exceeds packet limit of 1232"
    );

    // Process the transaction.
    context
        .banks_client
        .process_transaction(create_tx)
        .await
        .unwrap_or_else(|err| {
            panic!(
                "Creation error {:?}, create_rule_set_on_chain called at {}:{}:{}",
                err, file, line, column
            )
        });

    rule_set_addr
}

#[macro_export]
macro_rules! process_passing_validate_ix {
    ($context:expr, $validate_ix:expr, $additional_signers:expr, $compute_budget:expr) => {
        $crate::utils::process_passing_validate_ix_with_loc(
            $context,
            $validate_ix,
            $additional_signers,
            $compute_budget,
            file!(),
            line!(),
            column!(),
        )
    };
}

pub async fn process_passing_validate_ix_with_loc(
    context: &mut ProgramTestContext,
    validate_ix: Instruction,
    additional_signers: Vec<&Keypair>,
    compute_budget: Option<u32>,
    file: &str,
    line: u32,
    column: u32,
) {
    let mut signing_keypairs = vec![&context.payer];
    signing_keypairs.extend(additional_signers);

    // Use user-provided compute budget if one was provided.
    let instructions = match compute_budget {
        Some(units) => {
            let compute_budget_ix = ComputeBudgetInstruction::set_compute_unit_limit(units);
            vec![compute_budget_ix, validate_ix]
        }
        None => vec![validate_ix],
    };

    // Add ix to a transaction.
    let validate_tx = Transaction::new_signed_with_payer(
        &instructions,
        Some(&context.payer.pubkey()),
        &signing_keypairs,
        context.last_blockhash,
    );

    // Process the transaction.
    context
        .banks_client
        .process_transaction(validate_tx)
        .await
        .unwrap_or_else(|err| {
            panic!(
                "Validation error {:?}, process_passing_validate_ix called at {}:{}:{}",
                err, file, line, column
            )
        });
}

#[macro_export]
macro_rules! process_failing_validate_ix {
    ($context:expr, $validate_ix:expr, $additional_signers:expr, $compute_budget:expr) => {
        $crate::utils::process_failing_validate_ix_with_loc(
            $context,
            $validate_ix,
            $additional_signers,
            $compute_budget,
            file!(),
            line!(),
            column!(),
        )
    };
}

pub async fn process_failing_validate_ix_with_loc(
    context: &mut ProgramTestContext,
    validate_ix: Instruction,
    additional_signers: Vec<&Keypair>,
    compute_budget: Option<u32>,
    file: &str,
    line: u32,
    column: u32,
) -> BanksClientError {
    let mut signing_keypairs = vec![&context.payer];
    signing_keypairs.extend(additional_signers);

    // Use user-provided compute budget if one was provided.
    let instructions = match compute_budget {
        Some(units) => {
            let compute_budget_ix = ComputeBudgetInstruction::set_compute_unit_limit(units);
            vec![compute_budget_ix, validate_ix]
        }
        None => vec![validate_ix],
    };

    // Add ix to a transaction.
    let validate_tx = Transaction::new_signed_with_payer(
        &instructions,
        Some(&context.payer.pubkey()),
        &signing_keypairs,
        context.last_blockhash,
    );

    // Process the transaction.
    context
        .banks_client
        .process_transaction(validate_tx)
        .await
        .expect_err(&format!(
            "validation should fail, process_failing_validate_ix called at {}:{}:{}",
            file, line, column
        ))
}

#[macro_export]
macro_rules! assert_custom_error {
    ($error:expr, $matcher:pat) => {
        let calling_location = format!(
            "assert_custom_error called at {}:{}:{}",
            file!(),
            line!(),
            column!()
        );

        match $error {
            solana_program_test::BanksClientError::TransactionError(
                solana_sdk::transaction::TransactionError::InstructionError(
                    0,
                    solana_program::instruction::InstructionError::Custom(x),
                ),
            ) => match num_traits::FromPrimitive::from_i32(x as i32) {
                Some($matcher) => assert!(true),
                Some(other) => {
                    assert!(
                        false,
                        "Expected another custom instruction error than '{:#?}', {}",
                        other, calling_location
                    )
                }
                None => assert!(
                    false,
                    "Expected custom instruction error, {}",
                    calling_location
                ),
            },
            err => assert!(
                false,
                "Expected custom instruction error but got '{:#?}', {}",
                err, calling_location
            ),
        };
    };
}

pub async fn create_mint(
    context: &mut ProgramTestContext,
    mint: &Keypair,
    manager: &Pubkey,
    freeze_authority: Option<&Pubkey>,
    decimals: u8,
) -> Result<(), BanksClientError> {
    let rent = context.banks_client.get_rent().await.unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[
            system_instruction::create_account(
                &context.payer.pubkey(),
                &mint.pubkey(),
                rent.minimum_balance(spl_token::state::Mint::LEN),
                spl_token::state::Mint::LEN as u64,
                &spl_token::id(),
            ),
            spl_token::instruction::initialize_mint(
                &spl_token::id(),
                &mint.pubkey(),
                manager,
                freeze_authority,
                decimals,
            )
            .unwrap(),
        ],
        Some(&context.payer.pubkey()),
        &[&context.payer, mint],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await
}

pub async fn create_associated_token_account(
    context: &mut ProgramTestContext,
    wallet: &Keypair,
    token_mint: &Pubkey,
) -> Result<Pubkey, BanksClientError> {
    let recent_blockhash = context.last_blockhash;

    let tx = Transaction::new_signed_with_payer(
        &[
            spl_associated_token_account::instruction::create_associated_token_account(
                &context.payer.pubkey(),
                &wallet.pubkey(),
                token_mint,
                &spl_token::ID,
            ),
        ],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        recent_blockhash,
    );

    // connection.send_and_confirm_transaction(&tx)?;
    context.banks_client.process_transaction(tx).await.unwrap();

    Ok(spl_associated_token_account::get_associated_token_address(
        &wallet.pubkey(),
        token_mint,
    ))
}

pub fn cmp_slice<T: PartialEq>(a: &[T], b: &[T]) -> bool {
    let matching = a.iter().zip(b.iter()).filter(|&(a, b)| a == b).count();
    matching == a.len() && matching == b.len()
}

pub struct MerkleTree {
    pub root: [u8; 32],
    pub proof: ProofInfo,
}

pub fn create_test_merkle_tree_from_one_leaf(leaf: &Pubkey, levels: usize) -> MerkleTree {
    // Start hash with caller's leaf.
    let mut computed_hash = leaf.to_bytes();

    // Start proof with another random leaf.
    let mut proof = vec![Keypair::new().pubkey().to_bytes()];

    for i in 0..levels {
        if computed_hash <= proof[i] {
            // Hash(current computed hash + current element of the proof).
            computed_hash = solana_program::keccak::hashv(&[&[0x01], &computed_hash, &proof[i]]).0;
        } else {
            // Hash(current element of the proof + current computed hash).
            computed_hash = solana_program::keccak::hashv(&[&[0x01], &proof[i], &computed_hash]).0;
        }

        proof.push(computed_hash)
    }

    // The last hash value is the root.
    let root = proof.pop().unwrap();

    MerkleTree {
        root,
        proof: ProofInfo::new(proof),
    }
}
