use crate::payload::Payload;
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
    /// RuleSet pre-serialized by caller into the MessagePack format.
    pub serialized_rule_set: Vec<u8>,
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
/// Args for `validate` instruction.
pub struct ValidateArgs {
    /// `Operation` to validate.
    pub operation: String,
    /// `Payload` data used for rule validation.
    pub payload: Payload,
    /// Update any relevant state stored in Rule, such as the Frequency `last_update` time value.
    pub update_rule_state: bool,
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
    #[account(0, name="rule_set_pda", desc = "The PDA account where the RuleSet is stored")]
    #[account(1, writable, name="rule_set_state_pda", desc = "The PDA account where any RuleSet state is stored")]
    #[account(2, name="mint", desc="Mint of token asset")]
    #[account(3, name = "system_program", desc = "System program")]
    Validate(ValidateArgs),
}
/// Builds a `create` instruction.
pub fn create(
    payer: Pubkey,
    rule_set_pda: Pubkey,
    serialized_rule_set: Vec<u8>,
    additional_rule_accounts: Vec<Pubkey>,
) -> Instruction {
    let mut accounts = vec![
        AccountMeta::new(payer, true),
        AccountMeta::new(rule_set_pda, false),
        AccountMeta::new_readonly(solana_program::system_program::id(), false),
    ];

    for account in additional_rule_accounts {
        accounts.push(AccountMeta::new_readonly(account, false));
    }

    Instruction {
        program_id: crate::ID,
        accounts,
        data: RuleSetInstruction::Create(CreateArgs {
            serialized_rule_set,
        })
        .try_to_vec()
        .unwrap(),
    }
}

/// Builds a `validate` instruction.
#[allow(clippy::too_many_arguments)]
pub fn validate(
    rule_set_pda: Pubkey,
    rule_set_state_pda: Pubkey,
    mint: Pubkey,
    operation: String,
    payload: Payload,
    update_rule_state: bool,
    additional_rule_accounts: Vec<AccountMeta>,
) -> Instruction {
    let mut accounts = vec![
        AccountMeta::new_readonly(rule_set_pda, false),
        AccountMeta::new_readonly(rule_set_state_pda, false),
        AccountMeta::new(mint, false),
        AccountMeta::new_readonly(solana_program::system_program::id(), false),
    ];

    accounts.extend(additional_rule_accounts);

    Instruction {
        program_id: crate::ID,
        accounts,
        data: RuleSetInstruction::Validate(ValidateArgs {
            operation,
            payload,
            update_rule_state,
        })
        .try_to_vec()
        .unwrap(),
    }
}
