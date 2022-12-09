use crate::{state::Operation, Payload};
use borsh::{BorshDeserialize, BorshSerialize};
use shank::ShankInstruction;
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
/// Args for `create` instruction.
pub struct CreateArgs {
    /// Name of the RuleSet, used in PDA derivation.
    pub rule_set_name: String,
    /// RuleSet pre-serialized by caller into the MessagePack format.
    pub serialized_rule_set: Vec<u8>,
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
/// Args for `validate` instruction.
pub struct ValidateArgs {
    /// Name of the RuleSet, used in PDA derivation.
    pub rule_set_name: String,
    /// `Operation` to validate.
    pub operation: Operation,
    /// `Payload` data used for rule validation.
    pub payload: Payload,
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
/// Args for `create_frequency_rule` instruction.
pub struct CreateFrequencyRuleArgs {
    /// Name of the RuleSet, used in PDA derivation.
    pub rule_set_name: String,
    /// Name of the Frequency Rule, used in Frequency PDA derivation.
    pub freq_rule_name: String,
    /// Timestamp of last update.
    pub last_update: i64,
    /// Timestamp of permitted period.
    pub period: i64,
}

#[derive(Debug, Clone, ShankInstruction, BorshSerialize, BorshDeserialize)]
#[rustfmt::skip]
/// Instructions available in this program.
pub enum RuleSetInstruction {
    /// This instruction stores a caller-pre-serialized `RuleSet` into the rule_set PDA account.
    #[account(0, writable, signer, name="payer", desc="Payer and creator of the RuleSet")]
    #[account(1, writable, name="rule_set_pda", desc = "The PDA account where the RuleSet is stored")]
    #[account(2, name = "system_program", desc = "System program")]
    Create(CreateArgs),

    /// This instruction executes the RuleSet stored in the rule_set PDA account by sending
    /// it an `AccountsMap` and a `PayloadMap` and calling the `RuleSet`'s `validate` method.
    #[account(0, writable, signer, name="payer", desc="Payer and creator of the RuleSet")]
    #[account(1, writable, name="rule_set", desc = "The PDA account where the RuleSet is stored")]
    #[account(2, name = "system_program", desc = "System program")]
    #[account(3, optional, signer, name="opt_rule_signer_1", desc = "Optional rule validation signer 1")]
    #[account(4, optional, signer, name="opt_rule_signer_2", desc = "Optional rule validation signer 2")]
    #[account(5, optional, signer, name="opt_rule_signer_3", desc = "Optional rule validation signer 3")]
    #[account(6, optional, signer, name="opt_rule_signer_4", desc = "Optional rule validation signer 4")]
    #[account(7, optional, signer, name="opt_rule_signer_5", desc = "Optional rule validation signer 5")]
    #[account(8, optional, name = "opt_rule_nonsigner_1", desc = "Optional rule validation non-signer 1")]
    #[account(9, optional, name = "opt_rule_nonsigner_2", desc = "Optional rule validation non-signer 2")]
    #[account(10, optional, name = "opt_rule_nonsigner_3", desc = "Optional rule validation non-signer 3")]
    #[account(11, optional, name = "opt_rule_nonsigner_4", desc = "Optional rule validation non-signer 4")]
    #[account(12, optional, name = "opt_rule_nonsigner_5", desc = "Optional rule validation non-signer 5")]
    Validate(ValidateArgs),

    /// This instruction stores a Frequency Rule into a Frequency Rule PDA account.
    #[account(0, writable, signer, name="payer", desc="Payer and creator of the Frequency Rule")]
    #[account(1, writable, name="frequency_pda", desc = "The PDA account where the Frequency Rule is stored")]
    #[account(2, name = "system_program", desc = "System program")]
    CreateFrequencyRule(CreateFrequencyRuleArgs),
}
/// Builds a `create` instruction.
pub fn create(
    program_id: Pubkey,
    payer: Pubkey,
    rule_set_pda: Pubkey,
    rule_set_name: String,
    serialized_rule_set: Vec<u8>,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(payer, true),
        AccountMeta::new(rule_set_pda, false),
        AccountMeta::new_readonly(solana_program::system_program::id(), false),
    ];

    Instruction {
        program_id,
        accounts,
        data: RuleSetInstruction::Create(CreateArgs {
            rule_set_name,
            serialized_rule_set,
        })
        .try_to_vec()
        .unwrap(),
    }
}

/// Builds a `validate` instruction.
#[allow(clippy::too_many_arguments)]
pub fn validate(
    program_id: Pubkey,
    payer: Pubkey,
    rule_set_pda: Pubkey,
    rule_set_name: String,
    operation: Operation,
    payload: Payload,
    rule_signer_accounts: Vec<Pubkey>,
    rule_nonsigner_accounts: Vec<Pubkey>,
) -> Instruction {
    let mut accounts = vec![
        AccountMeta::new(payer, true),
        AccountMeta::new(rule_set_pda, false),
        AccountMeta::new_readonly(solana_program::system_program::id(), false),
    ];

    for i in 0..5 {
        if let Some(account) = rule_signer_accounts.get(i) {
            accounts.push(AccountMeta::new_readonly(*account, true));
        }
    }

    if rule_signer_accounts.get(5).is_some() {
        panic!("Too many rule validation signer accounts");
    }

    for i in 0..5 {
        if let Some(account) = rule_nonsigner_accounts.get(i) {
            accounts.push(AccountMeta::new_readonly(*account, false));
        }
    }

    if rule_nonsigner_accounts.get(5).is_some() {
        panic!("Too many rule validation non-signer accounts");
    }

    Instruction {
        program_id,
        accounts,
        data: RuleSetInstruction::Validate(ValidateArgs {
            rule_set_name,
            operation,
            payload,
        })
        .try_to_vec()
        .unwrap(),
    }
}

/// Builds a `create_frequency_rule` instruction.
pub fn create_frequency_rule(
    program_id: Pubkey,
    payer: Pubkey,
    freq_rule_pda: Pubkey,
    rule_set_name: String,
    freq_rule_name: String,
    last_update: i64,
    period: i64,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(payer, true),
        AccountMeta::new(freq_rule_pda, false),
        AccountMeta::new_readonly(solana_program::system_program::id(), false),
    ];

    Instruction {
        program_id,
        accounts,
        data: RuleSetInstruction::CreateFrequencyRule(CreateFrequencyRuleArgs {
            rule_set_name,
            freq_rule_name,
            last_update,
            period,
        })
        .try_to_vec()
        .unwrap(),
    }
}
