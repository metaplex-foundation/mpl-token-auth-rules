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

#[derive(Debug, Clone, ShankInstruction, BorshSerialize, BorshDeserialize)]
#[rustfmt::skip]
/// Instructions available in this program.
pub enum RuleSetInstruction {
    /// This instruction stores a caller-pre-serialized `RuleSet` into the rule_set PDA account.
    #[account(0, signer, writable, name="payer", desc="Payer and creator of the RuleSet")]
    #[account(1, writable, name="rule_set_pda", desc = "The PDA account where the RuleSet is stored")]
    #[account(2, name = "system_program", desc = "System program")]
    Create(CreateArgs),

    /// This instruction executes the RuleSet stored in the rule_set PDA account by calling the
    /// `RuleSet`'s `validate` method.  If any of the Rules contained in the RuleSet have state
    /// information (such as the Frequency rule's `last_update` time value), the optional accounts
    /// must be provided in order to save the updated stated in the RuleSet state PDA.  Note that
    /// updating the state for a Rule requires that the `rule_authority` signer matches the Pubkey
    /// stored in the Rule.
    #[account(0, name="rule_set_pda", desc = "The PDA account where the RuleSet is stored")]
    #[account(1, name="mint", desc="Mint of token asset")]
    #[account(2, name = "system_program", desc = "System program")]
    #[account(3, optional, signer, writable, name="payer", desc="Payer for RuleSet state PDA account")]
    #[account(4, optional, signer, name="rule_authority", desc="Signing authority for any Rule state updates")]
    #[account(5, optional, writable, name="rule_set_state_pda", desc = "The PDA account where any RuleSet state is stored")]
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
    mint: Pubkey,
    payer: Option<Pubkey>,
    rule_authority: Option<Pubkey>,
    rule_set_state_pda: Option<Pubkey>,
    operation: String,
    payload: Payload,
    update_rule_state: bool,
    additional_rule_accounts: Vec<AccountMeta>,
) -> Instruction {
    let mut accounts = vec![
        AccountMeta::new_readonly(rule_set_pda, false),
        AccountMeta::new_readonly(mint, false),
        AccountMeta::new_readonly(solana_program::system_program::id(), false),
    ];

    if let Some(payer) = payer {
        accounts.push(AccountMeta::new(payer, true));
    }

    if let Some(rule_authority) = rule_authority {
        accounts.push(AccountMeta::new_readonly(rule_authority, true));
    }

    if let Some(rule_set_state_pda) = rule_set_state_pda {
        accounts.push(AccountMeta::new(rule_set_state_pda, false));
    }

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
