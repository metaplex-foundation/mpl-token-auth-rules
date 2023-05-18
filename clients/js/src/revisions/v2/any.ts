import { Context, Serializer } from '@metaplex-foundation/umi';
import { RuleV2, getRuleV2Serializer } from './rule';
import { wrapSerializerInRuleHeaderV2 } from './ruleHeader';
import { RuleTypeV2 } from './ruleType';

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
