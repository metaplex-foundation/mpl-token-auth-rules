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

export type AnyRuleV2 = {
  type: 'Any';
  rules: RuleV2[];
};

export const anyV2 = (rules: RuleV2[]): AnyRuleV2 => ({ type: 'Any', rules });

export const getAnyRuleV2Serializer = (): Serializer<AnyRuleV2> =>
  wrapSerializerInRuleHeaderV2(
    RuleTypeV2.Any,
    struct([['rules', array(getRuleV2Serializer(), { size: u64() })]])
  );

export const isAnyRuleV2 = (rule: RuleV1 | RuleV2): rule is AnyRuleV2 =>
  isRuleV2(rule) && rule.type === 'Any';
