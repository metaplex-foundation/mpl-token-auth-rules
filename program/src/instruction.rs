use borsh::{BorshDeserialize, BorshSerialize};
use shank::ShankInstruction;
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub struct CreateArgs {
    pub name: String,
}

#[derive(Debug, Clone, ShankInstruction, BorshSerialize, BorshDeserialize)]
#[rustfmt::skip]
pub enum RuleSetInstruction {
    /// Description of this instruction
    #[account(0, writable, signer, name="payer", desc="Payer and creator of the rule set")]
    #[account(1, writable, name="ruleset", desc = "The PDA account where the ruleset is stored")]
    #[account(2, name = "system_program", desc = "System program")]
    Create(CreateArgs),
}

pub fn create(program_id: Pubkey, payer: Pubkey, ruleset: Pubkey, name: String) -> Instruction {
    let accounts = vec![
        AccountMeta::new(payer, true),
        AccountMeta::new(ruleset, false),
        AccountMeta::new_readonly(solana_program::system_program::id(), false),
    ];

    Instruction {
        program_id,
        accounts,
        data: RuleSetInstruction::Create(CreateArgs { name })
            .try_to_vec()
            .unwrap(),
    }
}
