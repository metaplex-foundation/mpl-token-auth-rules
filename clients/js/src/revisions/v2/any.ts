import { Context, Serializer } from '@metaplex-foundation/umi';
import { RuleV2, getRuleV2Serializer, isRuleV2 } from './rule';
import { wrapSerializerInRuleHeaderV2 } from './ruleHeader';
import { RuleTypeV2 } from './ruleType';
import { RuleV1 } from '../v1';

export type AnyRuleV2 = {
  type: 'Any';
  rules: RuleV2[];
};

export const anyV2 = (rules: RuleV2[]): AnyRuleV2 => ({ type: 'Any', rules });

export const getAnyRuleV2Serializer = (
  context: Pick<Context, 'serializer'>
): Serializer<AnyRuleV2> => {
  const s = context.serializer;
  return wrapSerializerInRuleHeaderV2(
    context,
    RuleTypeV2.Any,
    s.struct([
      ['rules', s.array(getRuleV2Serializer(context), { size: s.u64() })],
    ])
  );
};

export const isAnyRuleV2 = (rule: RuleV1 | RuleV2): rule is AnyRuleV2 =>
  isRuleV2(rule) && rule.type === 'Any';
