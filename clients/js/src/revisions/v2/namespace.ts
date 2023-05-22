import { Context, Serializer } from '@metaplex-foundation/umi';
import { wrapSerializerInRuleHeaderV2 } from './ruleHeader';
import { RuleTypeV2 } from './ruleType';
import { RuleV1 } from '../v1';
import { RuleV2, isRuleV2 } from './rule';

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

export const isNamespaceRuleV2 = (
  rule: RuleV1 | RuleV2
): rule is NamespaceRuleV2 => isRuleV2(rule) && rule.type === 'Namespace';
