import { Context, Serializer } from '@metaplex-foundation/umi';
import type { RuleV1 } from '../v1';
import { RuleV2, isRuleV2 } from './rule';
import { wrapSerializerInRuleHeaderV2 } from './ruleHeader';
import { RuleTypeV2 } from './ruleType';

export type PassRuleV2 = { type: 'Pass' };

export const passV2 = (): PassRuleV2 => ({ type: 'Pass' });

export const getPassRuleV2Serializer = (
  context: Pick<Context, 'serializer'>
): Serializer<PassRuleV2> => {
  const s = context.serializer;
  return wrapSerializerInRuleHeaderV2(context, RuleTypeV2.Pass, s.struct([]));
};

export const isPassRuleV2 = (rule: RuleV1 | RuleV2): rule is PassRuleV2 =>
  isRuleV2(rule) && rule.type === 'Pass';
