use crate::{
    error::RuleSetError,
    payload::{Payload, PayloadKey},
    utils::assert_derivation,
};
use serde::{Deserialize, Serialize};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey,
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
    /// An additional signer must be present.
    AdditionalSigner {
        /// The public key that must have also signed the transaction.
        account: Pubkey,
    },
    /// Direct comparison between Pubkeys.
    PubkeyMatch {
        /// The public key to be compared against.
        pubkey: Pubkey,
        /// The field in the `Payload` to be compared.
        field: PayloadKey,
    },
    /// The comparing `Pubkey` must be in the list of `Pubkey`s.
    PubkeyListMatch {
        /// The list of public keys to be compared against.
        pubkeys: Vec<Pubkey>,
        /// The field in the `Payload` to be compared.
        field: PayloadKey,
    },
    /// The pubkey must be a member of the Merkle tree.
    PubkeyTreeMatch {
        /// The root of the Merkle tree.
        root: [u8; 32],
        /// The field in the `Payload` to be compared.
        field: PayloadKey,
    },
    /// A resulting derivation of seeds must match to a `Pubkey`.
    DerivedKeyMatch {
        /// The program for which the derivation must be true.
        program: Pubkey,
        /// The field in the `Payload` to be compared.
        field: PayloadKey,
    },
    /// The `Pubkey` must be owned by a given program.
    ProgramOwned {
        /// The program that must own the `Pubkey`.
        program: Pubkey,
        /// The field in the `Payload` to be compared.
        field: PayloadKey,
    },
    /// Comparison against the amount of tokens being transferred.
    Amount {
        /// The amount to be compared against.
        amount: u64,
        /// The operator to be used in the comparison.
        operator: CompareOp,
    },
    /// Comparison based on time between operations.
    Frequency {
        /// The authority of the frequency account.
        authority: Pubkey,
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
        _rule_authority: &Option<&AccountInfo>,
    ) -> (bool, ProgramError) {
        match self {
            Rule::All { rules } => {
                msg!("Validating All");
                let mut last = self.to_error();
                for rule in rules {
                    last = rule.to_error();
                    let result = rule.low_level_validate(
                        accounts,
                        payload,
                        _update_rule_state,
                        _rule_set_state_pda,
                        _rule_authority,
                    );
                    if !result.0 {
                        return result;
                    }
                }
                (true, last)
            }
            Rule::Any { rules } => {
                msg!("Validating Any");
                let mut last = self.to_error();
                for rule in rules {
                    last = rule.to_error();
                    let result = rule.low_level_validate(
                        accounts,
                        payload,
                        _update_rule_state,
                        _rule_set_state_pda,
                        _rule_authority,
                    );
                    if result.0 {
                        return result;
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
                    _rule_authority,
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
            Rule::PubkeyTreeMatch { root, field } => {
                msg!("Validating PubkeyTreeMatch");

                let merkle_proof = match payload.get_merkle_proof(field) {
                    Some(merkle_proof) => merkle_proof,
                    _ => return (false, RuleSetError::MissingPayloadValue.into()),
                };

                let mut computed_hash = merkle_proof.leaf;
                for proof_element in merkle_proof.proof.iter() {
                    if computed_hash <= *proof_element {
                        // Hash(current computed hash + current element of the proof)
                        computed_hash = solana_program::keccak::hashv(&[
                            &[0x01],
                            &computed_hash,
                            proof_element,
                        ])
                        .0;
                    } else {
                        // Hash(current element of the proof + current computed hash)
                        computed_hash = solana_program::keccak::hashv(&[
                            &[0x01],
                            proof_element,
                            &computed_hash,
                        ])
                        .0;
                    }
                }
                // Check if the computed hash (root) is equal to the provided root
                if computed_hash == *root {
                    (true, self.to_error())
                } else {
                    (false, self.to_error())
                }
            }
            Rule::DerivedKeyMatch { program, field } => {
                msg!("Validating DerivedKeyMatch");

                let (account, seeds) = match payload.get_account_and_seeds(field) {
                    Some(seeds) => seeds,
                    _ => return (false, RuleSetError::MissingPayloadValue.into()),
                };

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
            Rule::Amount {
                amount: rule_amount,
                operator,
            } => {
                msg!("Validating Amount");
                if let Some(payload_amount) = &payload.get_amount(&PayloadKey::Amount) {
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

                if let Some(account) = accounts.get(authority) {
                    if !account.is_signer {
                        return (false, RuleSetError::RuleAuthorityIsNotSigner.into());
                    }
                } else {
                    return (false, RuleSetError::MissingAccount.into());
                }

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
                RuleSetError::NotImplemented.into()
            }
            Rule::AdditionalSigner { .. } => RuleSetError::AdditionalSignerCheckFailed.into(),
            Rule::PubkeyMatch { .. } => RuleSetError::PubkeyMatchCheckFailed.into(),
            Rule::PubkeyListMatch { .. } => RuleSetError::PubkeyListMatchCheckFailed.into(),
            Rule::PubkeyTreeMatch { .. } => RuleSetError::PubkeyTreeMatchCheckFailed.into(),
            Rule::DerivedKeyMatch { .. } => RuleSetError::DerivedKeyMatchCheckFailed.into(),
            Rule::ProgramOwned { .. } => RuleSetError::ProgramOwnedCheckFailed.into(),
            Rule::Amount { .. } => RuleSetError::AmountCheckFailed.into(),
            Rule::Frequency { .. } => RuleSetError::FrequencyCheckFailed.into(),
        }
    }
}
