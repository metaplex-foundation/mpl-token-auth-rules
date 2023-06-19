import { Serializer, struct } from '@metaplex-foundation/umi/serializers';
import type { RuleV1 } from '../v1';
import { RuleV2, getRuleV2Serializer, isRuleV2 } from './rule';
import { wrapSerializerInRuleHeaderV2 } from './ruleHeader';
import { RuleTypeV2 } from './ruleType';

export type NotRuleV2 = {
  type: 'Not';
  rule: RuleV2;
};

export const notV2 = (rule: RuleV2): NotRuleV2 => ({ type: 'Not', rule });

export const getNotRuleV2Serializer = (): Serializer<NotRuleV2> =>
  wrapSerializerInRuleHeaderV2(
    RuleTypeV2.Not,
    struct([['rule', getRuleV2Serializer()]])
  );

export const isNotRuleV2 = (rule: RuleV1 | RuleV2): rule is NotRuleV2 =>
  isRuleV2(rule) && rule.type === 'Not';
