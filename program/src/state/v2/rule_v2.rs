use bytemuck::{Pod, Zeroable};
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};
use std::collections::HashMap;

use crate::{
    error::RuleSetError,
    payload::Payload,
    state::{constraint::*, Constraint, ConstraintType, RuleResult, U64_BYTES},
    types::Assertable,
};

use super::try_from_bytes;

/// Size (in bytes) of the header section.
pub const HEADER_SECTION: usize = U64_BYTES;

/// Macro to automate the code required to deserialize a constraint from a byte array.
macro_rules! constraint_from_bytes {
    ( $constraint_type:ident, $slice:expr, $( $available:ident ),+ $(,)? ) => {
        match $constraint_type {
            $(
                $crate::state::ConstraintType::$available => {
                    Box::new($available::from_bytes($slice)?) as Box<dyn Constraint>
                }
            )+
            _ => return Err(RuleSetError::InvalidConstraintType),
        }
    };
}

/// Struct representing a 'RuleV2'.
///
/// A rule is a combination of a header and a constraint.
pub struct RuleV2<'a> {
    /// Header of the rule.
    pub header: &'a Header,
    /// Constraint represented by the rule.
    pub constraint: Box<dyn Constraint<'a> + 'a>,
}

impl<'a> RuleV2<'a> {
    /// Deserialize a constraint from a byte array.
    pub fn from_bytes(bytes: &'a [u8]) -> Result<Self, RuleSetError> {
        let (header, data) = bytes.split_at(HEADER_SECTION);
        let header = try_from_bytes::<Header>(0, HEADER_SECTION, header)?;

        let constraint_type = header.constraint_type();
        let length = header.length();

        let constraint = constraint_from_bytes!(
            constraint_type,
            &data[..length],
            AdditionalSigner,
            All,
            Amount,
            Any,
            Frequency,
            IsWallet,
            Namespace,
            Not,
            Pass,
            PDAMatch,
            ProgramOwnedList,
            ProgramOwnedTree,
            ProgramOwned,
            PubkeyListMatch,
            PubkeyMatch,
            PubkeyTreeMatch
        );

        Ok(Self { header, constraint })
    }

    /// Length (in bytes) of the serialized rule.
    pub fn length(&self) -> usize {
        HEADER_SECTION + self.header.length()
    }
}

impl<'a> Assertable<'a> for RuleV2<'a> {
    fn validate(
        &self,
        accounts: &HashMap<Pubkey, &AccountInfo>,
        payload: &Payload,
        update_rule_state: bool,
        rule_set_state_pda: &Option<&AccountInfo>,
        rule_authority: &Option<&AccountInfo>,
    ) -> ProgramResult {
        let result = self.constraint.validate(
            accounts,
            payload,
            update_rule_state,
            rule_set_state_pda,
            rule_authority,
        );

        match result {
            RuleResult::Success(_) => Ok(()),
            RuleResult::Failure(err) => Err(err),
            RuleResult::Error(err) => Err(err),
        }
    }
}

impl<'a> Constraint<'a> for RuleV2<'a> {
    fn constraint_type(&self) -> ConstraintType {
        self.constraint.constraint_type()
    }

    fn validate(
        &self,
        accounts: &std::collections::HashMap<
            solana_program::pubkey::Pubkey,
            &solana_program::account_info::AccountInfo,
        >,
        payload: &crate::payload::Payload,
        update_rule_state: bool,
        rule_set_state_pda: &Option<&solana_program::account_info::AccountInfo>,
        rule_authority: &Option<&solana_program::account_info::AccountInfo>,
    ) -> RuleResult {
        self.constraint.validate(
            accounts,
            payload,
            update_rule_state,
            rule_set_state_pda,
            rule_authority,
        )
    }
}

/// Header for the rule.
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Header {
    /// Header data.
    pub data: [u32; 2],
}

impl Header {
    /// Returns the type of the constraint.
    pub fn constraint_type(&self) -> ConstraintType {
        ConstraintType::try_from(self.data[0]).unwrap()
    }

    /// Returns the length of the data section.
    pub fn length(&self) -> usize {
        self.data[1] as usize
    }

    /// Serialize the header.
    pub fn serialize(constraint_type: ConstraintType, length: u32, data: &mut Vec<u8>) {
        // constraint type
        data.extend(u32::to_le_bytes(constraint_type as u32));
        // length
        data.extend(u32::to_le_bytes(length));
    }
}

#[cfg(test)]
mod tests {
    use super::RuleV2;
    use crate::state::v2::{Amount, Any, Operator, ProgramOwnedList, Str32};
    use solana_program::pubkey::Pubkey;

    #[test]
    fn test_create_amount() {
        let amount = Amount::serialize(String::from("Destination"), Operator::Eq, 1).unwrap();

        // loads the data using bytemuck

        let rule = RuleV2::from_bytes(&amount).unwrap();

        assert_eq!(rule.header.length(), 48);
    }

    #[test]
    fn test_create_program_owned_list() {
        let programs = &[Pubkey::default(), Pubkey::default()];

        let program_owned =
            ProgramOwnedList::serialize(String::from("Destination"), programs).unwrap();

        // loads the data using bytemuck

        let rule = RuleV2::from_bytes(&program_owned).unwrap();

        assert_eq!(rule.header.length(), 96);
    }

    #[test]
    fn test_create_large_program_owned_list() {
        const SIZE: usize = 1000;

        let mut programs = Vec::new();

        for _ in 0..SIZE {
            programs.push(Pubkey::default());
        }

        let program_owned =
            ProgramOwnedList::serialize(String::from("Destination"), programs.as_mut_slice())
                .unwrap();

        // loads the data using bytemuck

        let rule = RuleV2::from_bytes(&program_owned).unwrap();

        assert_eq!(rule.header.length(), Str32::SIZE + (SIZE * 32));
    }

    #[test]
    fn test_create_any() {
        let programs_list1 = &[Pubkey::default()];
        let program_owned1 =
            ProgramOwnedList::serialize(String::from("Destination"), programs_list1).unwrap();

        let programs_list2 = &[Pubkey::default(), Pubkey::default(), Pubkey::default()];
        let program_owned2 =
            ProgramOwnedList::serialize(String::from("Destination"), programs_list2).unwrap();

        let any = Any::serialize(&[&program_owned1, &program_owned2]).unwrap();

        // loads the data using bytemuck
        let rule = RuleV2::from_bytes(&any).unwrap();

        assert_eq!(
            rule.header.length(),
            8 + program_owned1.len() + program_owned2.len()
        );
    }
}
