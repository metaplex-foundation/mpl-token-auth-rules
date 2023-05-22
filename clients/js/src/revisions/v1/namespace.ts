import { RuleV2 } from "../v2";
import { RuleV1, isRuleV1 } from "./rule";

export type NamespaceRuleV1 = 'Namespace';

export const isNamespaceRuleV1 = (
  rule: RuleV1 | RuleV2
): rule is NamespaceRuleV1 => isRuleV1(rule) && rule === 'Namespace';
