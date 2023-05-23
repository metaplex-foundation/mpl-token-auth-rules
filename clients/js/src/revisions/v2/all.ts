import { Context, Serializer } from '@metaplex-foundation/umi';
import type { RuleV1 } from '../v1';
import { RuleV2, getRuleV2Serializer, isRuleV2 } from './rule';
import { wrapSerializerInRuleHeaderV2 } from './ruleHeader';
import { RuleTypeV2 } from './ruleType';

export type AllRuleV2 = {
  type: 'All';
  rules: RuleV2[];
};

export const allV2 = (rules: RuleV2[]): AllRuleV2 => ({ type: 'All', rules });

export const getAllRuleV2Serializer = (
  context: Pick<Context, 'serializer'>
): Serializer<AllRuleV2> => {
  const s = context.serializer;
  return wrapSerializerInRuleHeaderV2(
    context,
    RuleTypeV2.All,
    s.struct([
      ['rules', s.array(getRuleV2Serializer(context), { size: s.u64() })],
    ])
  );
};

export const isAllRuleV2 = (rule: RuleV1 | RuleV2): rule is AllRuleV2 =>
  isRuleV2(rule) && rule.type === 'All';
