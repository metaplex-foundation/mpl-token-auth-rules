use bytemuck::{Pod, Zeroable};
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};
use std::collections::HashMap;

use super::{All, Amount, Any, Constraint, ConstraintType, ProgramOwnedList, U64_BYTES};
use crate::{error::RuleSetError, payload::Payload, state::RuleResult, types::Assertable};

/// Size (in bytes) of the header section.
pub const HEADER_SECTION: usize = U64_BYTES;

/// Struct representing a 'RuleV2'.
///
/// A rule is a combination of a header and a constraint.
pub struct RuleV2<'a> {
    /// Header of the rule.
    pub header: &'a Header,
    /// Constraint represented by the rule.
    pub data: Box<dyn Constraint<'a> + 'a>,
}

impl<'a> RuleV2<'a> {
    /// Deserialize a constraint from a byte array.
    pub fn from_bytes(bytes: &'a [u8]) -> Result<Self, RuleSetError> {
        let (header, data) = bytes.split_at(HEADER_SECTION);
        let header = bytemuck::from_bytes::<Header>(header);

        let condition_type = header.constraint_type();
        let length = header.length();

        let data = match condition_type {
            ConstraintType::Amount => {
                Box::new(Amount::from_bytes(&data[..length])?) as Box<dyn Constraint>
            }
            ConstraintType::Any => {
                Box::new(Any::from_bytes(&data[..length])?) as Box<dyn Constraint>
            }
            ConstraintType::All => {
                Box::new(All::from_bytes(&data[..length])?) as Box<dyn Constraint>
            }
            ConstraintType::ProgramOwnedList => {
                Box::new(ProgramOwnedList::from_bytes(&data[..length])?) as Box<dyn Constraint>
            }
            _ => unimplemented!("condition type not implemented"),
        };

        Ok(Self { header, data })
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
        let result = self.data.validate(
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
        self.data.constraint_type()
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
        self.data.validate(
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
}

#[cfg(test)]
mod tests {
    use super::RuleV2;
    use crate::state::v2::{Amount, Any, Operator, ProgramOwnedList, Str32};
    use solana_program::pubkey::Pubkey;

    #[test]
    fn test_create_amount() {
        let amount = Amount::serialize(1, Operator::Eq, String::from("Destination")).unwrap();

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
