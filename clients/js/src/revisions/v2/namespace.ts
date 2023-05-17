import { Context, Serializer } from '@metaplex-foundation/umi';
import { wrapSerializerInRuleHeaderV2 } from './ruleHeader';
import { RuleTypeV2 } from './ruleType';

export type NamespaceRuleV2 = { type: 'Namespace' };

export const namespaceV2 = (): NamespaceRuleV2 => ({ type: 'Namespace' });

export const getNamespaceRuleV2Serializer = (
  context: Pick<Context, 'serializer'>
): Serializer<NamespaceRuleV2> => {
  const s = context.serializer;
  return wrapSerializerInRuleHeaderV2(
    context,
    RuleTypeV2.Namespace,
    s.struct([])
  );
};
