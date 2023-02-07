use crate::payload::Payload;
use borsh::{BorshDeserialize, BorshSerialize};
use mpl_token_metadata_context_derive::AccountContext;
use shank::ShankInstruction;
use solana_program::{
    account_info::AccountInfo,
    instruction::{AccountMeta, Instruction},
};

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
/// Args for `create` instruction.
pub enum CreateOrUpdateArgs {
    /// V1 implementation of the `create` instruction arguments.
    V1 {
        /// RuleSet pre-serialized by caller into the MessagePack format.
        serialized_rule_set: Vec<u8>,
    },
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
/// Args for `validate` instruction.
pub enum ValidateArgs {
    /// V1 implementation of the `validate` instruction arguments.
    V1 {
        /// `Operation` to validate.
        operation: String,
        /// `Payload` data used for rule validation.
        payload: Payload,
        /// Update any relevant state stored in Rule, such as the Frequency `last_update` time value.
        update_rule_state: bool,
        /// Optional revision of the `RuleSet` to use.  If `None`, the latest revision is used.
        rule_set_revision: Option<usize>,
    },
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
/// Args for `append_to_rule_set` instruction.
pub enum WriteToBufferArgs {
    /// V1 implementation of the `create` instruction arguments.
    V1 {
        /// RuleSet pre-serialized by caller into the MessagePack format.
        serialized_rule_set: Vec<u8>,
        /// Whether the or not the any old data should be overwritten.
        overwrite: bool,
    },
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
/// Args for `append_to_rule_set` instruction.
pub enum PuffRuleSetArgs {
    /// V1 implementation of the `create` instruction arguments.
    V1 {
        /// RuleSet name.
        rule_set_name: String,
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
    #[account(3, optional, name="buffer_pda", desc = "The buffer to copy a complete ruleset from")]
    #[default_optional_accounts]
    CreateOrUpdate(CreateOrUpdateArgs),

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
    #[args(additional_rule_accounts: Vec<AccountMeta>)]
    #[default_optional_accounts]
    Validate(ValidateArgs),

    /// This instruction appends a pre-serialized `RuleSet` chunk into the rule_set PDA account.
    /// Needed with large `RuleSet`s to stay within transaction size limit.
    #[account(0, signer, writable, name="payer", desc="Payer and creator of the RuleSet")]
    #[account(1, writable, name="buffer_pda", desc = "The PDA account where the RuleSet buffer is stored")]
    #[account(2, name = "system_program", desc = "System program")]
    WriteToBuffer(WriteToBufferArgs),

    /// Add space to the end of a rule set account.  Needed with large `RuleSet`s to pre-allocate
    /// the space, to stay within PDA allocation limits.
    #[account(0, signer, writable, name="payer", desc="Payer and creator of the RuleSet")]
    #[account(1, writable, name="rule_set_pda", desc = "The PDA account where the RuleSet is stored")]
    #[account(2, name = "system_program", desc = "System program")]
    PuffRuleSet(PuffRuleSetArgs),
}

/// Builds a `CreateOrUpdate` instruction.
impl InstructionBuilder for builders::CreateOrUpdate {
    fn instruction(&self) -> solana_program::instruction::Instruction {
        let mut accounts = vec![
            AccountMeta::new(self.payer, true),
            AccountMeta::new(self.rule_set_pda, false),
            AccountMeta::new_readonly(solana_program::system_program::id(), false),
        ];

        if let Some(buffer_pda) = self.buffer_pda {
            accounts.push(AccountMeta::new_readonly(buffer_pda, false));
        } else {
            accounts.push(AccountMeta::new_readonly(crate::ID, false));
        }

        Instruction {
            program_id: crate::ID,
            accounts,
            data: RuleSetInstruction::CreateOrUpdate(self.args.clone())
                .try_to_vec()
                .unwrap(),
        }
    }
}

/// Builds a `Validate` instruction.
impl InstructionBuilder for builders::Validate {
    fn instruction(&self) -> solana_program::instruction::Instruction {
        let mut accounts = vec![
            AccountMeta::new_readonly(self.rule_set_pda, false),
            AccountMeta::new_readonly(self.mint, false),
            AccountMeta::new_readonly(solana_program::system_program::id(), false),
        ];

        // Add optional account or `crate::ID`.
        if let Some(payer) = self.payer {
            accounts.push(AccountMeta::new(payer, true));
        } else {
            accounts.push(AccountMeta::new_readonly(crate::ID, false));
        }

        // Add optional account or `crate::ID`.
        if let Some(rule_authority) = self.rule_authority {
            accounts.push(AccountMeta::new_readonly(rule_authority, true));
        } else {
            accounts.push(AccountMeta::new_readonly(crate::ID, false));
        }

        // Add optional account or `crate::ID`.
        if let Some(rule_set_state_pda) = self.rule_set_state_pda {
            accounts.push(AccountMeta::new(rule_set_state_pda, false));
        } else {
            accounts.push(AccountMeta::new_readonly(crate::ID, false));
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

/// Builds a `WriteToBuffer` instruction.
impl InstructionBuilder for builders::WriteToBuffer {
    fn instruction(&self) -> solana_program::instruction::Instruction {
        let accounts = vec![
            AccountMeta::new(self.payer, true),
            AccountMeta::new(self.buffer_pda, false),
            AccountMeta::new_readonly(solana_program::system_program::id(), false),
        ];

        Instruction {
            program_id: crate::ID,
            accounts,
            data: RuleSetInstruction::WriteToBuffer(self.args.clone())
                .try_to_vec()
                .unwrap(),
        }
    }
}

/// Builds a `PuffRuleSet` instruction.
impl InstructionBuilder for builders::PuffRuleSet {
    fn instruction(&self) -> solana_program::instruction::Instruction {
        let accounts = vec![
            AccountMeta::new(self.payer, true),
            AccountMeta::new(self.rule_set_pda, false),
            AccountMeta::new_readonly(solana_program::system_program::id(), false),
        ];

        Instruction {
            program_id: crate::ID,
            accounts,
            data: RuleSetInstruction::PuffRuleSet(self.args.clone())
                .try_to_vec()
                .unwrap(),
        }
    }
}

/// Account context holding the accounts used by various instructions.
pub struct Context<'a, T> {
    /// The struct holding the named accounts used by an instruction.
    pub accounts: T,
    /// All remaining accounts passed to an instruction.
    pub remaining_accounts: Vec<&'a AccountInfo<'a>>,
}

/// A trait for building an instruction.
pub trait InstructionBuilder {
    /// The required function to return the built instruction.
    fn instruction(&self) -> solana_program::instruction::Instruction;
}
