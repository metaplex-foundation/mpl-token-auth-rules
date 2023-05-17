import { Context, Serializer } from '@metaplex-foundation/umi';
import { RuleV2, getRuleV2Serializer } from './rule';
import { wrapSerializerInRuleHeaderV2 } from './ruleHeader';
import { RuleTypeV2 } from './ruleType';

export type NotRuleV2 = {
  type: 'Not';
  rule: RuleV2;
};

export const notV2 = (rule: RuleV2): NotRuleV2 => ({ type: 'Not', rule });

export const getNotRuleV2Serializer = (
  context: Pick<Context, 'serializer'>
): Serializer<NotRuleV2> => {
  const s = context.serializer;
  return wrapSerializerInRuleHeaderV2(
    context,
    RuleTypeV2.Not,
    s.struct([['rule', getRuleV2Serializer(context)]])
  );
};
