import { Serializer, struct } from '@metaplex-foundation/umi/serializers';
import type { RuleV1 } from '../v1';
import { RuleV2, isRuleV2 } from './rule';
import { wrapSerializerInRuleHeaderV2 } from './ruleHeader';
import { RuleTypeV2 } from './ruleType';

export type PassRuleV2 = { type: 'Pass' };

export const passV2 = (): PassRuleV2 => ({ type: 'Pass' });

export const getPassRuleV2Serializer = (): Serializer<PassRuleV2> =>
  wrapSerializerInRuleHeaderV2(RuleTypeV2.Pass, struct([]));

export const isPassRuleV2 = (rule: RuleV1 | RuleV2): rule is PassRuleV2 =>
  isRuleV2(rule) && rule.type === 'Pass';
