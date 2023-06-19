import {
  Serializer,
  array,
  struct,
  u64,
} from '@metaplex-foundation/umi/serializers';
import type { RuleV1 } from '../v1';
import { RuleV2, getRuleV2Serializer, isRuleV2 } from './rule';
import { wrapSerializerInRuleHeaderV2 } from './ruleHeader';
import { RuleTypeV2 } from './ruleType';

export type AllRuleV2 = {
  type: 'All';
  rules: RuleV2[];
};

export const allV2 = (rules: RuleV2[]): AllRuleV2 => ({ type: 'All', rules });

export const getAllRuleV2Serializer = (): Serializer<AllRuleV2> =>
  wrapSerializerInRuleHeaderV2(
    RuleTypeV2.All,
    struct([['rules', array(getRuleV2Serializer(), { size: u64() })]])
  );

export const isAllRuleV2 = (rule: RuleV1 | RuleV2): rule is AllRuleV2 =>
  isRuleV2(rule) && rule.type === 'All';
