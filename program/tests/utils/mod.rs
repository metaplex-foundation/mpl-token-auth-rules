use mpl_token_auth_rules::{
    error::RuleSetError,
    instruction::{
        builders::{CreateOrUpdateBuilder, WriteToBufferBuilder},
        CreateOrUpdateArgs, InstructionBuilder, WriteToBufferArgs,
    },
    state::RuleSetV1,
};
use num_derive::ToPrimitive;
use num_traits::cast::FromPrimitive;
use rmp_serde::Serializer;
use serde::Serialize;
use solana_program::{
    instruction::{Instruction, InstructionError},
    program_error::ProgramError,
    pubkey::Pubkey,
};
use solana_program_test::{BanksClientError, ProgramTest, ProgramTestContext};
use solana_sdk::{
    compute_budget::ComputeBudgetInstruction,
    program_pack::Pack,
    signature::Signer,
    signer::keypair::Keypair,
    system_instruction,
    transaction::{Transaction, TransactionError},
};

#[repr(C)]
#[derive(ToPrimitive)]
pub enum Operation {
    OwnerTransfer,
    Delegate,
    SaleTransfer,
}

impl ToString for Operation {
    fn to_string(&self) -> String {
        match self {
            Operation::OwnerTransfer => "OwnerTransfer".to_string(),
            Operation::Delegate => "Delegate".to_string(),
            Operation::SaleTransfer => "SaleTransfer".to_string(),
        }
    }
}

#[repr(C)]
#[derive(ToPrimitive)]
pub enum PayloadKey {
    /// The amount being transferred.
    Amount,
    /// The authority of a transfer, e.g. the delegate of token.
    Authority,
    /// Seeds for a PDA authority of the operation, e.g. when the authority is a PDA.
    AuthoritySeeds,
    /// Merkle proof for the source of the operation, e.g. when the authority is a member
    /// of a Merkle tree.
    AuthorityProof,
    /// The source of the operation, e.g. the owner initiating a transfer.
    Source,
    /// Seeds for a PDA source of the operation, e.g. when the source is a PDA.
    SourceSeeds,
    /// Merkle proof for the source of the operation, e.g. when the source is a member
    /// of a Merkle tree.
    SourceProof,
    /// The destination of the operation, e.g. the recipient of a transfer.
    Destination,
    /// Seeds for a PDA destination of the operation, e.g. when the recipient is a PDA.
    DestinationSeeds,
    /// Merkle proof for the destination of the operation, e.g. when the distination
    /// is a member of a Merkle tree.
    DestinationProof,
}

impl ToString for PayloadKey {
    fn to_string(&self) -> String {
        match self {
            PayloadKey::Amount => "Amount".to_string(),
            PayloadKey::Authority => "Authority".to_string(),
            PayloadKey::AuthoritySeeds => "AuthoritySeeds".to_string(),
            PayloadKey::AuthorityProof => "AuthorityProof".to_string(),
            PayloadKey::Source => "Source".to_string(),
            PayloadKey::SourceSeeds => "SourceSeeds".to_string(),
            PayloadKey::SourceProof => "SourceProof".to_string(),
            PayloadKey::Destination => "Destination".to_string(),
            PayloadKey::DestinationSeeds => "DestinationSeeds".to_string(),
            PayloadKey::DestinationProof => "DestinationProof".to_string(),
        }
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
    ($context:expr, $rule_set:expr, $rule_set_name:expr) => {
        $crate::utils::create_big_rule_set_on_chain_with_loc(
            $context,
            $rule_set,
            $rule_set_name,
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
    file: &str,
    line: u32,
    column: u32,
) -> Pubkey {
    // Find RuleSet PDA.
    let (rule_set_addr, _rule_set_bump) =
        mpl_token_auth_rules::pda::find_rule_set_address(context.payer.pubkey(), rule_set_name);

    let (buffer_pda, _buffer_bump) =
        mpl_token_auth_rules::pda::find_buffer_address(context.payer.pubkey());

    // Serialize the RuleSet using RMP serde.
    let mut serialized_rule_set = Vec::new();
    rule_set
        .serialize(&mut Serializer::new(&mut serialized_rule_set))
        .unwrap();

    let mut overwrite = true;
    for serialized_rule_set_chunk in serialized_rule_set.chunks(1000) {
        // Create a `create` instruction.
        let create_ix = WriteToBufferBuilder::new()
            .payer(context.payer.pubkey())
            .buffer_pda(buffer_pda)
            .build(WriteToBufferArgs::V1 {
                serialized_rule_set: serialized_rule_set_chunk.to_vec(),
                overwrite,
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

        println!("TX Length: {:?}", create_tx.message.serialize().len());
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
macro_rules! assert_rule_set_error {
    ($err:path, $rule_set_error:path) => {
        $crate::utils::assert_rule_set_error_with_loc(
            $err,
            $rule_set_error,
            file!(),
            line!(),
            column!(),
        );
    };
}

pub fn assert_rule_set_error_with_loc(
    err: BanksClientError,
    rule_set_error: RuleSetError,
    file: &str,
    line: u32,
    column: u32,
) {
    let calling_location = format!(
        "assert_rule_set_error called at {}:{}:{}",
        file, line, column
    );
    // Deconstruct the error code and make sure it is what we expect.
    match err {
        BanksClientError::TransactionError(TransactionError::InstructionError(
            _,
            InstructionError::Custom(val),
        )) => {
            let deconstructed_err = RuleSetError::from_u32(val).unwrap();
            assert_eq!(deconstructed_err, rule_set_error, "{}", calling_location);
        }
        _ => panic!("Unexpected error {:?}, {}", err, calling_location),
    }
}

#[macro_export]
macro_rules! assert_program_error {
    ($err:path, $rule_set_error:path) => {
        $crate::utils::assert_program_error_with_loc(
            $err,
            $rule_set_error,
            file!(),
            line!(),
            column!(),
        )
    };
}

pub fn assert_program_error_with_loc(
    err: BanksClientError,
    program_error: ProgramError,
    file: &str,
    line: u32,
    column: u32,
) {
    let calling_location = format!(
        "assert_program_error called at {}:{}:{}",
        file, line, column
    );
    // Deconstruct the error code and make sure it is what we expect.
    match err {
        BanksClientError::TransactionError(TransactionError::InstructionError(_, err)) => {
            assert_eq!(
                ProgramError::try_from(err).unwrap_or_else(|_| panic!(
                    "Could not convert InstructionError to ProgramError at {}",
                    calling_location,
                )),
                program_error,
                "{}",
                calling_location,
            );
        }
        _ => panic!("Unexpected error {:?}, {}", err, calling_location),
    }
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
