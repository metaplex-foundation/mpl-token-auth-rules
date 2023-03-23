use borsh::BorshSerialize;
use std::fmt::Display;

use crate::{
    error::RuleSetError,
    state_v2::{Condition, ConditionType, RuleV2, HEADER_SECTION, U64_BYTES},
};

pub struct Any<'a> {
    pub size: &'a u64,
    pub rules: Vec<RuleV2<'a>>,
}

impl<'a> Any<'a> {
    pub fn from_bytes(bytes: &'a [u8]) -> Result<Self, RuleSetError> {
        let (size, data) = bytes.split_at(U64_BYTES);
        let size = bytemuck::from_bytes::<u64>(size);

        let mut rules = Vec::with_capacity(*size as usize);
        let mut offset = 0;

        for _ in 0..*size {
            let rule = RuleV2::from_bytes(&data[offset..])?;
            offset += rule.length();
            rules.push(rule);
        }

        Ok(Self { size, rules })
    }

    pub fn serialize(rules: &[&[u8]]) -> std::io::Result<Vec<u8>> {
        let length = (U64_BYTES
            + rules
                .iter()
                .map(|v| v.len())
                .reduce(|accum, item| accum + item)
                .ok_or(RuleSetError::DataIsEmpty)
                .unwrap()) as u32;

        let mut data = Vec::with_capacity(HEADER_SECTION + length as usize);

        // Header
        // - rule type
        let rule_type = ConditionType::Any as u32;
        BorshSerialize::serialize(&rule_type, &mut data)?;
        // - length
        BorshSerialize::serialize(&length, &mut data)?;

        // Assert
        // - size
        let size = rules.len() as u64;
        BorshSerialize::serialize(&size, &mut data)?;
        // - rules
        rules.iter().for_each(|x| data.extend(x.iter()));

        Ok(data)
    }
}

impl<'a> Condition<'a> for Any<'a> {
    fn condition_type(&self) -> ConditionType {
        ConditionType::Any
    }
}

impl<'a> Display for Any<'a> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("Any {rules: [")?;

        for i in 0..*self.size {
            if i > 0 {
                formatter.write_str(", ")?;
            }
            formatter.write_str(&format!("{}", self.rules[i as usize]))?;
        }

        formatter.write_str("]}")
    }
}
