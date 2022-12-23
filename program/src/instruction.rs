use crate::payload::Payload;
use borsh::{BorshDeserialize, BorshSerialize};
use mpl_token_metadata_context_derive::AccountContext;
use shank::ShankInstruction;
use solana_program::{
    account_info::AccountInfo,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
/// Args for `create` instruction.
pub enum CreateArgs {
    V1 {
        /// RuleSet pre-serialized by caller into the MessagePack format.
        serialized_rule_set: Vec<u8>,
    },
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
/// Args for `validate` instruction.

pub enum ValidateArgs {
    V1 {
        /// `Operation` to validate.
        operation: String,
        /// `Payload` data used for rule validation.
        payload: Payload,
        /// Update any relevant state stored in Rule, such as the Frequency `last_update` time value.
        update_rule_state: bool,
    },
}

#[derive(Debug, Clone, ShankInstruction, AccountContext, BorshSerialize, BorshDeserialize)]
#[rustfmt::skip]
/// Instructions available in this program.
pub enum RuleSetInstruction {
    /// This instruction stores a caller-pre-serialized `RuleSet` into the rule_set PDA account.
    #[account(0, signer, writable, name="payer", desc="Payer and creator of the RuleSet")]
    #[account(1, writable, name="rule_set_pda", desc = "The PDA account where the RuleSet is stored")]
    #[account(2, name = "system_program", desc = "System program")]
    #[args(serialized_rule_set: Vec<u8>)]
    #[args(additional_rule_accounts: Vec<Pubkey>)]
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
    #[args(operation: String)]
    #[args(payload: Payload)]
    #[args(update_rule_state: bool)]
    #[args(additional_rule_accounts: Vec<AccountMeta>)]
    Validate(ValidateArgs),
}

/// Builds a `create` instruction.
impl InstructionBuilder for builders::Create {
    fn instruction(&self) -> solana_program::instruction::Instruction {
        let mut accounts = vec![
            AccountMeta::new(self.payer, true),
            AccountMeta::new(self.rule_set_pda, false),
            AccountMeta::new_readonly(solana_program::system_program::id(), false),
        ];

        for account in self.additional_rule_accounts.iter() {
            accounts.push(AccountMeta::new_readonly(*account, false));
        }

        Instruction {
            program_id: crate::ID,
            accounts,
            data: RuleSetInstruction::Create(self.args.clone())
                .try_to_vec()
                .unwrap(),
        }
    }
}

/// Builds a `validate` instruction.
impl InstructionBuilder for builders::Validate {
    fn instruction(&self) -> solana_program::instruction::Instruction {
        let mut accounts = vec![
            AccountMeta::new_readonly(self.rule_set_pda, false),
            AccountMeta::new_readonly(self.mint, false),
            AccountMeta::new_readonly(solana_program::system_program::id(), false),
        ];

        if let Some(payer) = self.payer {
            accounts.push(AccountMeta::new(payer, true));
        }

        if let Some(rule_authority) = self.rule_authority {
            accounts.push(AccountMeta::new_readonly(rule_authority, true));
        }

        if let Some(rule_set_state_pda) = self.rule_set_state_pda {
            accounts.push(AccountMeta::new(rule_set_state_pda, false));
        }

        accounts.extend(self.additional_rule_accounts.clone());

        Instruction {
            program_id: crate::ID,
            accounts,
            data: RuleSetInstruction::Validate(self.args.clone())
                .try_to_vec()
                .unwrap(),
        }
    }
}

pub struct Context<'a, T> {
    pub accounts: T,
    pub remaining_accounts: Vec<&'a AccountInfo<'a>>,
}

pub trait InstructionBuilder {
    fn instruction(&self) -> solana_program::instruction::Instruction;
}
