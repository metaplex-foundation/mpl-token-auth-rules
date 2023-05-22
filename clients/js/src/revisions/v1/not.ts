import { RuleV2 } from '../v2';
import { RuleV1, isRuleV1 } from './rule';

export type NotRuleV1 = {
  Not: {
    rule: RuleV1;
  };
};

export const isNotRuleV1 = (rule: RuleV1 | RuleV2): rule is NotRuleV1 =>
  isRuleV1(rule) && 'Not' in (rule as object);
