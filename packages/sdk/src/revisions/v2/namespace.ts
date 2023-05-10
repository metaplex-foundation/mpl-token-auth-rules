import { serializeRuleHeaderV2 } from './rule';
import { RuleTypeV2 } from './ruleType';

export type NamespaceRuleV2 = { type: 'Namespace' };

export const namespaceV2 = (): NamespaceRuleV2 => ({ type: 'Namespace' });

// eslint-disable-next-line @typescript-eslint/no-unused-vars
export const serializeNamespaceV2 = (rule: NamespaceRuleV2): Buffer => {
  return serializeRuleHeaderV2(RuleTypeV2.Namespace, 0);
};

// eslint-disable-next-line @typescript-eslint/no-unused-vars
export const deserializeNamespaceV2 = (buffer: Buffer, offset = 0): NamespaceRuleV2 => {
  return namespaceV2();
};
