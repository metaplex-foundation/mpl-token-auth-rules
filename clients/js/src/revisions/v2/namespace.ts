import { Serializer, struct } from '@metaplex-foundation/umi/serializers';
import type { RuleV1 } from '../v1';
import { RuleV2, isRuleV2 } from './rule';
import { wrapSerializerInRuleHeaderV2 } from './ruleHeader';
import { RuleTypeV2 } from './ruleType';

export type NamespaceRuleV2 = { type: 'Namespace' };

export const namespaceV2 = (): NamespaceRuleV2 => ({ type: 'Namespace' });

export const getNamespaceRuleV2Serializer = (): Serializer<NamespaceRuleV2> =>
  wrapSerializerInRuleHeaderV2(RuleTypeV2.Namespace, struct([]));

export const isNamespaceRuleV2 = (
  rule: RuleV1 | RuleV2
): rule is NamespaceRuleV2 => isRuleV2(rule) && rule.type === 'Namespace';
