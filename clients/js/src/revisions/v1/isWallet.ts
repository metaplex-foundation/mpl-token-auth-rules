import type { RuleV2 } from '../v2';
import { RuleV1, isRuleV1 } from './rule';

export type IsWalletRuleV1 = {
  IsWallet: {
    field: string;
  };
};

export const isIsWalletRuleV1 = (
  rule: RuleV1 | RuleV2
): rule is IsWalletRuleV1 => isRuleV1(rule) && 'IsWallet' in (rule as object);
