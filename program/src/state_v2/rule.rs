use bytemuck::{Pod, Zeroable};
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};
use std::{collections::HashMap, fmt::Display};

use super::{All, Amount, Any, Condition, ConditionType, ProgramOwnedList};
use crate::{error::RuleSetError, payload::Payload, state::RuleResult, types::Assertable};

// Size of the header section.
pub const HEADER_SECTION: usize = 8;

pub struct RuleV2<'a> {
    pub header: &'a Header,
    pub data: Box<dyn Condition<'a> + 'a>,
}

impl<'a> RuleV2<'a> {
    pub fn from_bytes(bytes: &'a [u8]) -> Result<Self, RuleSetError> {
        let (header, data) = bytes.split_at(HEADER_SECTION);
        let header = bytemuck::from_bytes::<Header>(header);

        let condition_type = header.condition_type();
        let length = header.length();

        let data = match condition_type {
            ConditionType::Amount => {
                Box::new(Amount::from_bytes(&data[..length])?) as Box<dyn Condition>
            }
            ConditionType::Any => Box::new(Any::from_bytes(&data[..length])?) as Box<dyn Condition>,
            ConditionType::All => Box::new(All::from_bytes(&data[..length])?) as Box<dyn Condition>,
            ConditionType::ProgramOwnedList => {
                Box::new(ProgramOwnedList::from_bytes(&data[..length])?) as Box<dyn Condition>
            }
            _ => unimplemented!("condition type not implemented"),
        };

        Ok(Self { header, data })
    }

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

impl<'a> Condition<'a> for RuleV2<'a> {
    fn condition_type(&self) -> ConditionType {
        self.data.condition_type()
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

impl<'a> Display for RuleV2<'a> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_fmt(format_args!("{}", self.data))
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Header {
    pub data: [u32; 2],
}

impl Header {
    pub fn condition_type(&self) -> ConditionType {
        ConditionType::try_from(self.data[0]).unwrap()
    }

    pub fn length(&self) -> usize {
        self.data[1] as usize
    }
}

#[cfg(test)]
mod tests {
    use super::RuleV2;
    use crate::state_v2::{Amount, Any, CompareOp, ProgramOwnedList, Str32};
    use solana_program::pubkey::Pubkey;

    #[test]
    fn test_create_amount() {
        let amount = Amount::serialize(1, CompareOp::Eq, String::from("Destination")).unwrap();

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
        println!("{}", rule);

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

        println!("{}", rule);

        assert_eq!(
            rule.header.length(),
            8 + program_owned1.len() + program_owned2.len()
        );
    }
}
