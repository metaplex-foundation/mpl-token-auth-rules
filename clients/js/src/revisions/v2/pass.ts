import { Context, Serializer } from '@metaplex-foundation/umi';
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
