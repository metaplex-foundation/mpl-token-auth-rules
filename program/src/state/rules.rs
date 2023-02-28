use crate::{
    error::RuleSetError,
    payload::Payload,
    // TODO: Uncomment this after on-curve sycall available.
    // utils::is_on_curve,
    utils::{assert_derivation, compute_merkle_root, is_zeroed},
};
use serde::{Deserialize, Serialize};
#[cfg(feature = "serde-with-feature")]
use serde_with::{As, DisplayFromStr};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey, system_program,
};
use std::collections::{HashMap, HashSet};

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

/// Enum representation of Rule failure conditions
pub enum RuleResult {
    /// The rule succeeded.
    Success(ProgramError),
    /// The rule failed.
    Failure(ProgramError),
    /// The program failed to execute the rule.
    Error(ProgramError),
}

use RuleResult::*;

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
        #[cfg_attr(feature = "serde-with-feature", serde(with = "As::<DisplayFromStr>"))]
        account: Pubkey,
    },
    /// Direct comparison between `Pubkey`s.  When the `Validate` instruction is called, this rule
    /// requires a `PayloadType` value of `PayloadType::Pubkey`.  The `field` value in the rule is
    /// used to locate the `Pubkey` in the payload to compare to the `Pubkey` in the rule.
    PubkeyMatch {
        /// The public key to be compared against.
        #[cfg_attr(feature = "serde-with-feature", serde(with = "As::<DisplayFromStr>"))]
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
        #[cfg_attr(feature = "serde-with-feature", serde(with = "As::<DisplayFromStr>"))]
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
        #[cfg_attr(feature = "serde-with-feature", serde(with = "As::<DisplayFromStr>"))]
        authority: Pubkey,
    },
    /// The true test if a pubkey can be signed from a client and therefore is a true wallet account.
    /// The details of this rule are as follows: a wallet is defined as being both owned by the
    /// System Program and the address is on-curve.  The `field` value in the rule is used to
    /// locate the `Pubkey` in the payload that must be on-curve and for which the owner must be
    /// the System Program.  Note this same `Pubkey` account must also be provided to `Validate`
    /// via the `additional_rule_accounts` argument.  This is so that the `Pubkey`'s owner can be
    /// found from its `AccountInfo` struct.
    IsWallet {
        /// The field in the `Payload` to be checked.
        field: String,
    },
    /// An operation that always succeeds.
    Pass,
    /// The `Pubkey` must be owned by a program in the set of `Pubkey`s.  When the `Validate`
    /// instruction is called, this rule requires a `PayloadType` value of `PayloadType::Pubkey`.
    /// The `field` value in the rule is used to locate the `Pubkey` in the payload for which the
    /// owner must be a program in the set in the rule.  Note this same `Pubkey` account must also
    /// be provided to `Validate` via the `additional_rule_accounts` argument.  This is so that the
    /// `Pubkey`'s owner can be found from its `AccountInfo` struct.
    ProgramOwnedSet {
        /// The program that must own the `Pubkey`.
        programs: HashSet<Pubkey>,
        /// The field in the `Payload` to be compared.
        field: String,
    },
    /// A rule that tells the operation finder to use the default namespace rule.
    Namespace,
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
        let result = self.low_level_validate(
            accounts,
            payload,
            update_rule_state,
            rule_set_state_pda,
            rule_authority,
        );

        match result {
            Success(_) => Ok(()),
            Failure(err) => Err(err),
            Error(err) => Err(err),
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
    ) -> RuleResult {
        match self {
            Rule::All { rules } => {
                msg!("Validating All");
                let mut last: Option<ProgramError> = None;
                for rule in rules {
                    let result = rule.low_level_validate(
                        accounts,
                        payload,
                        _update_rule_state,
                        _rule_set_state_pda,
                        rule_authority,
                    );
                    // Return failure on the first failing rule.
                    match result {
                        Success(err) => last = Some(err),
                        _ => return result,
                    }
                }

                // Return pass if and only if all rules passed.
                Success(last.unwrap_or_else(|| RuleSetError::UnexpectedRuleSetFailure.into()))
            }
            Rule::Any { rules } => {
                msg!("Validating Any");
                let mut last_failure: Option<ProgramError> = None;
                let mut last_error: Option<ProgramError> = None;
                for rule in rules {
                    let result = rule.low_level_validate(
                        accounts,
                        payload,
                        _update_rule_state,
                        _rule_set_state_pda,
                        rule_authority,
                    );
                    match result {
                        Success(_) => return result,
                        Failure(err) => last_failure = Some(err),
                        Error(err) => last_error = Some(err),
                    }
                }

                // Return failure if and only if all rules failed.  Use the last failure.
                if let Some(err) = last_failure {
                    Failure(err)
                } else if let Some(err) = last_error {
                    // Return invalid if and only if all rules were invalid.  Use the last invalid.
                    Error(err)
                } else {
                    Error(RuleSetError::UnexpectedRuleSetFailure.into())
                }
            }
            Rule::Not { rule } => {
                let result = rule.low_level_validate(
                    accounts,
                    payload,
                    _update_rule_state,
                    _rule_set_state_pda,
                    rule_authority,
                );

                // Negate the result.
                match result {
                    Success(err) => Failure(err),
                    Failure(err) => Success(err),
                    Error(err) => Error(err),
                }
            }
            Rule::AdditionalSigner { account } => {
                msg!("Validating AdditionalSigner");
                if let Some(signer) = accounts.get(account) {
                    if signer.is_signer {
                        Success(self.to_error())
                    } else {
                        Failure(self.to_error())
                    }
                } else {
                    Error(RuleSetError::MissingAccount.into())
                }
            }
            Rule::PubkeyMatch { pubkey, field } => {
                msg!("Validating PubkeyMatch");

                let key = match payload.get_pubkey(field) {
                    Some(pubkey) => pubkey,
                    _ => return Error(RuleSetError::MissingPayloadValue.into()),
                };

                if key == pubkey {
                    Success(self.to_error())
                } else {
                    Failure(self.to_error())
                }
            }
            Rule::PubkeyListMatch { pubkeys, field } => {
                msg!("Validating PubkeyListMatch");

                let fields = field.split('|').collect::<Vec<&str>>();

                if fields.len() > 1 {
                    let new_rule = Rule::Any {
                        rules: fields
                            .iter()
                            .map(|field| Rule::ProgramOwnedList {
                                programs: pubkeys.clone(),
                                field: field.to_string(),
                            })
                            .collect(),
                    };

                    return new_rule.low_level_validate(
                        accounts,
                        payload,
                        _update_rule_state,
                        _rule_set_state_pda,
                        rule_authority,
                    );
                } else {
                    let key = match payload.get_pubkey(&field.to_owned()) {
                        Some(pubkey) => pubkey,
                        _ => return Error(RuleSetError::MissingPayloadValue.into()),
                    };

                    if pubkeys.iter().any(|pubkey| pubkey == key) {
                        return Success(self.to_error());
                    }
                }

                Failure(self.to_error())
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
                    _ => return Error(RuleSetError::MissingPayloadValue.into()),
                };

                // Get the Merkle proof from the payload.
                let merkle_proof = match payload.get_merkle_proof(proof_field) {
                    Some(merkle_proof) => merkle_proof,
                    _ => return Error(RuleSetError::MissingPayloadValue.into()),
                };

                // Check if the computed hash (root) is equal to the root in the rule.
                let computed_root = compute_merkle_root(leaf, merkle_proof);
                if computed_root == *root {
                    Success(self.to_error())
                } else {
                    Failure(self.to_error())
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
                    _ => return Error(RuleSetError::MissingPayloadValue.into()),
                };

                // Get the derivation seeds from the payload.
                let seeds = match payload.get_seeds(seeds_field) {
                    Some(seeds) => seeds,
                    _ => return Error(RuleSetError::MissingPayloadValue.into()),
                };

                // Get the program ID to use for the PDA derivation from the Rule.
                let program = match program {
                    // If the Pubkey is stored in the rule, use that value.
                    Some(program) => program,
                    None => {
                        // If one is not stored, then assume the program ID is the account owner.
                        match accounts.get(account) {
                            Some(account) => account.owner,
                            _ => return Error(RuleSetError::MissingAccount.into()),
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
                    Success(self.to_error())
                } else {
                    Failure(self.to_error())
                }
            }
            Rule::ProgramOwned { program, field } => {
                msg!("Validating ProgramOwned");

                let key = match payload.get_pubkey(field) {
                    Some(pubkey) => pubkey,
                    _ => return Error(RuleSetError::MissingPayloadValue.into()),
                };

                if let Some(account) = accounts.get(key) {
                    let data = match account.data.try_borrow() {
                        Ok(data) => data,
                        Err(_) => return Error(ProgramError::AccountBorrowFailed),
                    };

                    if is_zeroed(&data) {
                        // Print helpful errors.
                        if data.len() == 0 {
                            msg!("Account data is empty");
                        } else {
                            msg!("Account data is zeroed");
                        }

                        // Account must have nonzero data to count as program-owned.
                        return Error(self.to_error());
                    } else if *account.owner == *program {
                        return Success(self.to_error());
                    }
                } else {
                    return Error(RuleSetError::MissingAccount.into());
                }

                Failure(self.to_error())
            }
            Rule::ProgramOwnedList { programs, field } => {
                msg!("Validating ProgramOwnedList");

                let fields = field.split('|').collect::<Vec<&str>>();

                if fields.len() > 1 {
                    let new_rule = Rule::Any {
                        rules: fields
                            .iter()
                            .map(|field| Rule::ProgramOwnedList {
                                programs: programs.clone(),
                                field: field.to_string(),
                            })
                            .collect(),
                    };

                    return new_rule.low_level_validate(
                        accounts,
                        payload,
                        _update_rule_state,
                        _rule_set_state_pda,
                        rule_authority,
                    );
                } else {
                    let key = match payload.get_pubkey(&field.to_string()) {
                        Some(pubkey) => pubkey,
                        _ => return Error(RuleSetError::MissingPayloadValue.into()),
                    };

                    let account = match accounts.get(key) {
                        Some(account) => account,
                        _ => return Error(RuleSetError::MissingAccount.into()),
                    };

                    let data = match account.data.try_borrow() {
                        Ok(data) => data,
                        Err(_) => return Error(ProgramError::AccountBorrowFailed),
                    };

                    if is_zeroed(&data) {
                        // Print helpful errors.
                        if data.len() == 0 {
                            msg!("Account data is empty");
                        } else {
                            msg!("Account data is zeroed");
                        }

                        return Error(RuleSetError::DataIsEmpty.into());
                    } else if programs.contains(account.owner) {
                        // Account owner must be in the set.
                        return Success(self.to_error());
                    }
                }

                Failure(self.to_error())
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
                    _ => return Error(RuleSetError::MissingPayloadValue.into()),
                };

                // Get the `AccountInfo` struct for the `Pubkey`.
                let account = match accounts.get(key) {
                    Some(account) => account,
                    _ => return Error(RuleSetError::MissingAccount.into()),
                };

                let data = match account.data.try_borrow() {
                    Ok(data) => data,
                    Err(_) => return Error(ProgramError::AccountBorrowFailed),
                };

                // Account must have nonzero data to count as program-owned.
                if is_zeroed(&data) {
                    // Print helpful errors.
                    if data.len() == 0 {
                        msg!("Account data is empty");
                    } else {
                        msg!("Account data is zeroed");
                    }

                    return Error(RuleSetError::DataIsEmpty.into());
                }

                // The account owner is the leaf.
                let leaf = account.owner;

                // Get the Merkle proof from the payload.
                let merkle_proof = match payload.get_merkle_proof(proof_field) {
                    Some(merkle_proof) => merkle_proof,
                    _ => return Error(RuleSetError::MissingPayloadValue.into()),
                };

                // Check if the computed hash (root) is equal to the root in the rule.
                let computed_root = compute_merkle_root(leaf, merkle_proof);
                if computed_root == *root {
                    Success(self.to_error())
                } else {
                    Failure(self.to_error())
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
                        Success(self.to_error())
                    } else {
                        Failure(self.to_error())
                    }
                } else {
                    Error(RuleSetError::MissingPayloadValue.into())
                }
            }
            Rule::Frequency { authority } => {
                msg!("Validating Frequency");

                if let Some(rule_authority) = rule_authority {
                    // TODO: If it's the wrong account (first condition) the `IsNotASigner`
                    // is misleading.  Should be improved, perhaps with a `Mismatch` error.
                    if authority != rule_authority.key || !rule_authority.is_signer {
                        return Error(RuleSetError::RuleAuthorityIsNotSigner.into());
                    }
                } else {
                    return Error(RuleSetError::MissingAccount.into());
                }

                Error(RuleSetError::NotImplemented.into())
            }
            Rule::Pass => {
                msg!("Validating Pass");
                Success(self.to_error())
            }
            Rule::IsWallet { field } => {
                msg!("Validating IsWallet");

                // Get the `Pubkey` we are checking from the payload.
                let key = match payload.get_pubkey(field) {
                    Some(pubkey) => pubkey,
                    _ => return Error(RuleSetError::MissingPayloadValue.into()),
                };

                // Get the `AccountInfo` struct for the `Pubkey` and verify that
                // its owner is the System Program.
                if let Some(account) = accounts.get(key) {
                    if *account.owner != system_program::ID {
                        // TODO: Change error return to commented line after on-curve syscall
                        // available.
                        return Error(RuleSetError::NotImplemented.into());
                        //return (false, self.to_error());
                    }
                } else {
                    return Error(RuleSetError::MissingAccount.into());
                }

                // TODO: Uncomment call to `is_on_curve()` after on-curve sycall available.
                Error(RuleSetError::NotImplemented.into())
                //(is_on_curve(key), self.to_error())
            }
            Rule::ProgramOwnedSet { programs, field } => {
                msg!("Validating ProgramOwnedSet");

                let fields = field.split('|').collect::<Vec<&str>>();

                if fields.len() > 1 {
                    let new_rule = Rule::Any {
                        rules: fields
                            .iter()
                            .map(|field| Rule::ProgramOwnedSet {
                                programs: programs.clone(),
                                field: field.to_string(),
                            })
                            .collect(),
                    };

                    return new_rule.low_level_validate(
                        accounts,
                        payload,
                        _update_rule_state,
                        _rule_set_state_pda,
                        rule_authority,
                    );
                } else {
                    let key = match payload.get_pubkey(&field.to_string()) {
                        Some(pubkey) => pubkey,
                        _ => return Error(RuleSetError::MissingPayloadValue.into()),
                    };

                    let account = match accounts.get(key) {
                        Some(account) => account,
                        _ => return Error(RuleSetError::MissingAccount.into()),
                    };

                    let data = match account.data.try_borrow() {
                        Ok(data) => data,
                        Err(_) => return Error(ProgramError::AccountBorrowFailed),
                    };

                    if is_zeroed(&data) {
                        // Print helpful errors.
                        if data.len() == 0 {
                            msg!("Account data is empty");
                        } else {
                            msg!("Account data is zeroed");
                        }

                        return Error(RuleSetError::DataIsEmpty.into());
                    } else if programs.contains(account.owner) {
                        // Account owner must be in the set.
                        return Success(self.to_error());
                    }
                }

                Failure(self.to_error())
            }
            Rule::Namespace => {
                msg!("Validating Namespace");
                Failure(self.to_error())
            }
        }
    }

    /// Convert the rule to a corresponding error resulting from the rule failure.
    pub fn to_error(&self) -> ProgramError {
        match self {
            Rule::All { .. }
            | Rule::Any { .. }
            | Rule::Not { .. }
            | Rule::Pass
            | Rule::Namespace => RuleSetError::UnexpectedRuleSetFailure.into(),
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
            Rule::ProgramOwnedSet { .. } => RuleSetError::ProgramOwnedSetCheckFailed.into(),
        }
    }
}
