use crate::{
    error::RuleSetError,
    payload::Payload,
    utils::{assert_derivation, compute_merkle_root},
};
use serde::{Deserialize, Serialize};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey, system_program,
};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
/// Operators that can be used to compare against an `Amount` rule.
pub enum CompareOp {
    /// Less Than
    Lt,
    /// Less Than or Equal To
    LtEq,
    /// Equal To
    Eq,
    /// Greater Than or Equal To
    GtEq,
    /// Greater Than
    Gt,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
/// The struct containing every type of Rule and its associated data.
pub enum Rule {
    /// Group AND, where every rule contained must pass.
    All {
        /// The vector of Rules contained under All.
        rules: Vec<Rule>,
    },
    /// Group OR, where at least one rule contained must pass.
    Any {
        /// The vector of Rules contained under Any.
        rules: Vec<Rule>,
    },
    /// Negation, where the contained rule must fail.
    Not {
        /// The Rule contained under Not.
        rule: Box<Rule>,
    },
    /// An additional signer must be present.  When the `Validate` instruction is called, this rule
    /// does not require any `Payload` values, but the additional signer account must be provided
    /// to `Validate` via the `additional_rule_accounts` argument so that whether it is a signer
    /// can be retrieved from its `AccountInfo` struct.
    AdditionalSigner {
        /// The public key that must have also signed the transaction.
        account: Pubkey,
    },
    /// Direct comparison between `Pubkey`s.  When the `Validate` instruction is called, this rule
    /// requires a `PayloadType` value of `PayloadType::Pubkey`.  The `field` value in the rule is
    /// used to locate the `Pubkey` in the payload to compare to the `Pubkey` in the rule.
    PubkeyMatch {
        /// The public key to be compared against.
        pubkey: Pubkey,
        /// The field in the `Payload` to be compared.
        field: String,
    },
    /// The comparing `Pubkey` must be in the list of `Pubkey`s.  When the `Validate` instruction
    /// is called, this rule requires a `PayloadType` value of `PayloadType::Pubkey`.  The `field`
    /// value in the Rule is used to locate the `Pubkey` in the payload to compare to the `Pubkey`
    /// list in the rule.
    PubkeyListMatch {
        /// The list of public keys to be compared against.
        pubkeys: Vec<Pubkey>,
        /// The field in the `Payload` to be compared.
        field: String,
    },
    /// The comparing `Pubkey` must be a member of the Merkle tree in the rule.  When the
    /// `Validate` instruction is called, this rule requires `PayloadType` values of
    /// `PayloadType::Pubkey` and `PayloadType::MerkleProof`.  The `field` values in the Rule are
    /// used to locate them in the `Payload`.  The `Pubkey` and the proof are used to calculate
    /// a Merkle root which is compared against the root stored in the rule.
    PubkeyTreeMatch {
        /// The root of the Merkle tree.
        root: [u8; 32],
        /// The field in the `Payload` to be compared
        /// when looking for the `Pubkey`.
        pubkey_field: String,
        /// The field in the `Payload` to be compared
        /// when looking for the Merkle proof.
        proof_field: String,
    },
    /// A resulting PDA derivation of seeds must prove the account is a PDA.  When the `Validate`
    /// instruction is called, this rule requires `PayloadType` values of `PayloadType::Seeds`.
    /// The `field` values in the Rule are used to locate them in the `Payload`.  The seeds in the
    /// `Payload` and the program ID stored in the Rule are used to derive the PDA from the
    /// `Payload`.
    PDAMatch {
        /// The program used for the PDA derivation.  If
        /// `None` then the account owner is used.
        program: Option<Pubkey>,
        /// The field in the `Payload` to be compared
        /// when looking for the PDA.
        pda_field: String,
        /// The field in the `Payload` to be compared
        /// when looking for the seeds.
        seeds_field: String,
    },
    /// The `Pubkey` must be owned by a given program.  When the `Validate` instruction is called,
    /// this rule requires a `PayloadType` value of `PayloadType::Pubkey`.  The `field` value in
    /// the rule is used to locate the `Pubkey` in the payload for which the owner must be the
    /// program in the rule.  Note this same `Pubkey` account must also be provided to `Validate`
    /// via the `additional_rule_accounts` argument.  This is so that the `Pubkey`'s owner can be
    /// found from its `AccountInfo` struct.
    ProgramOwned {
        /// The program that must own the `Pubkey`.
        program: Pubkey,
        /// The field in the `Payload` to be compared.
        field: String,
    },
    /// The `Pubkey` must be owned by a program in the list of `Pubkey`s.  When the `Validate`
    /// instruction is called, this rule requires a `PayloadType` value of `PayloadType::Pubkey`.
    /// The `field` value in the rule is used to locate the `Pubkey` in the payload for which the
    /// owner must be a program in the list in the rule.  Note this same `Pubkey` account must also
    /// be provided to `Validate` via the `additional_rule_accounts` argument.  This is so that the
    /// `Pubkey`'s owner can be found from its `AccountInfo` struct.
    ProgramOwnedList {
        /// The program that must own the `Pubkey`.
        programs: Vec<Pubkey>,
        /// The field in the `Payload` to be compared.
        field: String,
    },
    /// The `Pubkey` must be owned by a member of the Merkle tree in the rule.  When the `Validate`
    /// instruction is called, this rule requires `PayloadType` values of `PayloadType::Pubkey` and
    /// `PayloadType::MerkleProof`.  The `field` values in the Rule are used to locate them in the
    /// `Payload`.  Note this same `Pubkey` account must also be provided to `Validate` via the
    /// `additional_rule_accounts` argument.  This is so that the `Pubkey`'s owner can be found
    /// from its `AccountInfo` struct.  The owner and the proof are then used to calculate a Merkle
    /// root, which is compared against the root stored in the rule.
    ProgramOwnedTree {
        /// The root of the Merkle tree.
        root: [u8; 32],
        /// The field in the `Payload` to be compared
        /// when looking for the `Pubkey`.
        pubkey_field: String,
        /// The field in the `Payload` to be compared
        /// when looking for the Merkle proof.
        proof_field: String,
    },
    /// Comparison against the amount of tokens being transferred.   When the `Validate`
    /// instruction is called, this rule requires a `PayloadType` value of `PayloadType::Amount`.
    /// The `field` value in the Rule is used to locate the numerical amount in the payload to
    /// compare to the amount stored in the rule, using the comparison operator stored in the rule.
    Amount {
        /// The amount to be compared against.
        amount: u64,
        /// The operator to be used in the comparison.
        operator: CompareOp,
        /// The field the amount is stored in.
        field: String,
    },
    /// Comparison based on time between operations.  Currently not implemented.  This rule
    /// is planned check to ensure a certain amount of time has passed.  This rule will make use
    /// of the `rule_set_state_pda` optional account passed into `Validate`, and will require
    /// the optional `rule_authority` account to sign.
    Frequency {
        /// The authority of the frequency account.
        authority: Pubkey,
    },
    /// A rule that checks whether an account is a wallet, where wallet is defined as being owned
    /// by the System Program and the address is on-curve (not a PDA).  The `field` value in
    /// the rule is used to locate the `Pubkey` in the payload that must be on-curve and for which
    /// the owner must be the System Program.  Note this same `Pubkey` account must also be provided
    /// to `Validate` via the `additional_rule_accounts` argument.  This is so that the `Pubkey`'s
    /// owner can be found from its `AccountInfo` struct.
    IsWallet {
        /// The field in the `Payload` to be checked.
        field: String,
    },
    /// An operation that always succeeds.
    Pass,
}

impl Rule {
    /// The top level validation function which parses an entire rule tree.
    pub fn validate(
        &self,
        accounts: &HashMap<Pubkey, &AccountInfo>,
        payload: &Payload,
        update_rule_state: bool,
        rule_set_state_pda: &Option<&AccountInfo>,
        rule_authority: &Option<&AccountInfo>,
    ) -> ProgramResult {
        let (status, rollup_err) = self.low_level_validate(
            accounts,
            payload,
            update_rule_state,
            rule_set_state_pda,
            rule_authority,
        );

        if status {
            ProgramResult::Ok(())
        } else {
            ProgramResult::Err(rollup_err)
        }
    }

    /// Lower level validation function which iterates through a rule tree and applies boolean logic to rule results.
    pub fn low_level_validate(
        &self,
        accounts: &HashMap<Pubkey, &AccountInfo>,
        payload: &Payload,
        _update_rule_state: bool,
        _rule_set_state_pda: &Option<&AccountInfo>,
        rule_authority: &Option<&AccountInfo>,
    ) -> (bool, ProgramError) {
        match self {
            Rule::All { rules } => {
                msg!("Validating All");
                for rule in rules {
                    let result = rule.low_level_validate(
                        accounts,
                        payload,
                        _update_rule_state,
                        _rule_set_state_pda,
                        rule_authority,
                    );
                    if !result.0 {
                        return result;
                    }
                }
                (true, self.to_error())
            }
            Rule::Any { rules } => {
                msg!("Validating Any");
                let mut last = self.to_error();
                for rule in rules {
                    let result = rule.low_level_validate(
                        accounts,
                        payload,
                        _update_rule_state,
                        _rule_set_state_pda,
                        rule_authority,
                    );
                    if result.0 {
                        return result;
                    } else {
                        last = result.1;
                    }
                }
                (false, last)
            }
            Rule::Not { rule } => {
                let result = rule.low_level_validate(
                    accounts,
                    payload,
                    _update_rule_state,
                    _rule_set_state_pda,
                    rule_authority,
                );
                (!result.0, result.1)
            }
            Rule::AdditionalSigner { account } => {
                msg!("Validating AdditionalSigner");
                if let Some(signer) = accounts.get(account) {
                    (signer.is_signer, self.to_error())
                } else {
                    (false, RuleSetError::MissingAccount.into())
                }
            }
            Rule::PubkeyMatch { pubkey, field } => {
                msg!("Validating PubkeyMatch");

                let key = match payload.get_pubkey(field) {
                    Some(pubkey) => pubkey,
                    _ => return (false, RuleSetError::MissingPayloadValue.into()),
                };

                if key == pubkey {
                    (true, self.to_error())
                } else {
                    (false, self.to_error())
                }
            }
            Rule::PubkeyListMatch { pubkeys, field } => {
                msg!("Validating PubkeyListMatch");

                let key = match payload.get_pubkey(field) {
                    Some(pubkey) => pubkey,
                    _ => return (false, RuleSetError::MissingPayloadValue.into()),
                };

                if pubkeys.iter().any(|pubkey| pubkey == key) {
                    (true, self.to_error())
                } else {
                    (false, self.to_error())
                }
            }
            Rule::PubkeyTreeMatch {
                root,
                pubkey_field,
                proof_field,
            } => {
                msg!("Validating PubkeyTreeMatch");

                // Get the `Pubkey` we are checking from the payload.
                let leaf = match payload.get_pubkey(pubkey_field) {
                    Some(pubkey) => pubkey,
                    _ => return (false, RuleSetError::MissingPayloadValue.into()),
                };

                // Get the Merkle proof from the payload.
                let merkle_proof = match payload.get_merkle_proof(proof_field) {
                    Some(merkle_proof) => merkle_proof,
                    _ => return (false, RuleSetError::MissingPayloadValue.into()),
                };

                // Check if the computed hash (root) is equal to the root in the rule.
                let computed_root = compute_merkle_root(leaf, merkle_proof);
                if computed_root == *root {
                    (true, self.to_error())
                } else {
                    (false, self.to_error())
                }
            }
            Rule::PDAMatch {
                program,
                pda_field,
                seeds_field,
            } => {
                msg!("Validating PDAMatch");

                // Get the PDA from the payload.
                let account = match payload.get_pubkey(pda_field) {
                    Some(pubkey) => pubkey,
                    _ => return (false, RuleSetError::MissingPayloadValue.into()),
                };

                // Get the derivation seeds from the payload.
                let seeds = match payload.get_seeds(seeds_field) {
                    Some(seeds) => seeds,
                    _ => return (false, RuleSetError::MissingPayloadValue.into()),
                };

                // Get the program ID to use for the PDA derivation from the Rule.
                let program = match program {
                    // If the Pubkey is stored in the rule, use that value.
                    Some(program) => program,
                    None => {
                        // If one is not stored, then assume the program ID is the account owner.
                        match accounts.get(account) {
                            Some(account) => account.owner,
                            _ => return (false, RuleSetError::MissingAccount.into()),
                        }
                    }
                };

                // Convert the Vec of Vec into Vec of u8 slices.
                let vec_of_slices = seeds
                    .seeds
                    .iter()
                    .map(Vec::as_slice)
                    .collect::<Vec<&[u8]>>();

                if let Ok(_bump) = assert_derivation(program, account, &vec_of_slices) {
                    (true, self.to_error())
                } else {
                    (false, self.to_error())
                }
            }
            Rule::ProgramOwned { program, field } => {
                msg!("Validating ProgramOwned");

                let key = match payload.get_pubkey(field) {
                    Some(pubkey) => pubkey,
                    _ => return (false, RuleSetError::MissingPayloadValue.into()),
                };

                if let Some(account) = accounts.get(key) {
                    if *account.owner == *program {
                        return (true, self.to_error());
                    }
                } else {
                    return (false, RuleSetError::MissingAccount.into());
                }

                (false, self.to_error())
            }
            Rule::ProgramOwnedList { programs, field } => {
                msg!("Validating ProgramOwnedList");

                let key = match payload.get_pubkey(field) {
                    Some(pubkey) => pubkey,
                    _ => return (false, RuleSetError::MissingPayloadValue.into()),
                };

                let account = match accounts.get(key) {
                    Some(account) => account,
                    _ => return (false, RuleSetError::MissingAccount.into()),
                };

                if programs.iter().any(|program| *account.owner == *program) {
                    (true, self.to_error())
                } else {
                    (false, self.to_error())
                }
            }
            Rule::ProgramOwnedTree {
                root,
                pubkey_field,
                proof_field,
            } => {
                msg!("Validating ProgramOwnedTree");

                // Get the `Pubkey` we are checking from the payload.
                let key = match payload.get_pubkey(pubkey_field) {
                    Some(pubkey) => pubkey,
                    _ => return (false, RuleSetError::MissingPayloadValue.into()),
                };

                // Get the `AccountInfo` struct for the `Pubkey`.
                let account = match accounts.get(key) {
                    Some(account) => account,
                    _ => return (false, RuleSetError::MissingAccount.into()),
                };

                // The account owner is the leaf.
                let leaf = account.owner;

                // Get the Merkle proof from the payload.
                let merkle_proof = match payload.get_merkle_proof(proof_field) {
                    Some(merkle_proof) => merkle_proof,
                    _ => return (false, RuleSetError::MissingPayloadValue.into()),
                };

                // Check if the computed hash (root) is equal to the root in the rule.
                let computed_root = compute_merkle_root(leaf, merkle_proof);
                if computed_root == *root {
                    (true, self.to_error())
                } else {
                    (false, self.to_error())
                }
            }
            Rule::Amount {
                amount: rule_amount,
                operator,
                field,
            } => {
                msg!("Validating Amount");
                if let Some(payload_amount) = &payload.get_amount(field) {
                    let operator_fn = match operator {
                        CompareOp::Lt => PartialOrd::lt,
                        CompareOp::LtEq => PartialOrd::le,
                        CompareOp::Eq => PartialEq::eq,
                        CompareOp::Gt => PartialOrd::gt,
                        CompareOp::GtEq => PartialOrd::ge,
                    };

                    if operator_fn(payload_amount, rule_amount) {
                        (true, self.to_error())
                    } else {
                        (false, self.to_error())
                    }
                } else {
                    (false, RuleSetError::MissingPayloadValue.into())
                }
            }
            Rule::Frequency { authority } => {
                msg!("Validating Frequency");

                if let Some(rule_authority) = rule_authority {
                    if authority != rule_authority.key || !rule_authority.is_signer {
                        return (false, RuleSetError::RuleAuthorityIsNotSigner.into());
                    }
                } else {
                    return (false, RuleSetError::MissingAccount.into());
                }

                (false, RuleSetError::NotImplemented.into())
            }
            Rule::IsWallet { field } => {
                msg!("Validating IsWallet");

                // Get the `Pubkey` we are checking from the payload.
                let key = match payload.get_pubkey(field) {
                    Some(pubkey) => pubkey,
                    _ => return (false, RuleSetError::MissingPayloadValue.into()),
                };

                // Get the `AccountInfo` struct for the `Pubkey` and verify that
                // its owner is the System Program.
                if let Some(account) = accounts.get(key) {
                    if *account.owner != system_program::ID {
                        return (false, self.to_error());
                    }
                } else {
                    return (false, RuleSetError::MissingAccount.into());
                }

                // TODO: Implement on-curve check here.
                (false, RuleSetError::NotImplemented.into())
            }
            Rule::Pass => {
                msg!("Validating Pass");
                (true, self.to_error())
            }
        }
    }

    /// Convert the rule to a corresponding error resulting from the rule failure.
    pub fn to_error(&self) -> ProgramError {
        match self {
            Rule::All { .. } | Rule::Any { .. } | Rule::Not { .. } | Rule::Pass => {
                RuleSetError::UnexpectedRuleSetFailure.into()
            }
            Rule::AdditionalSigner { .. } => RuleSetError::AdditionalSignerCheckFailed.into(),
            Rule::PubkeyMatch { .. } => RuleSetError::PubkeyMatchCheckFailed.into(),
            Rule::PubkeyListMatch { .. } => RuleSetError::PubkeyListMatchCheckFailed.into(),
            Rule::PubkeyTreeMatch { .. } => RuleSetError::PubkeyTreeMatchCheckFailed.into(),
            Rule::PDAMatch { .. } => RuleSetError::PDAMatchCheckFailed.into(),
            Rule::ProgramOwned { .. } => RuleSetError::ProgramOwnedCheckFailed.into(),
            Rule::ProgramOwnedList { .. } => RuleSetError::ProgramOwnedListCheckFailed.into(),
            Rule::ProgramOwnedTree { .. } => RuleSetError::ProgramOwnedTreeCheckFailed.into(),
            Rule::Amount { .. } => RuleSetError::AmountCheckFailed.into(),
            Rule::Frequency { .. } => RuleSetError::FrequencyCheckFailed.into(),
            Rule::IsWallet { .. } => RuleSetError::IsWalletCheckFailed.into(),
        }
    }
}
