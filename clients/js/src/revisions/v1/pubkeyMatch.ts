import type { RuleV2 } from '../v2';
import type { PublicKeyAsArrayOfBytes } from './publicKey';
import { RuleV1, isRuleV1 } from './rule';

export type PubkeyMatchRuleV1 = {
  PubkeyMatch: {
    pubkey: PublicKeyAsArrayOfBytes;
    field: string;
  };
};

export const isPubkeyMatchRuleV1 = (
  rule: RuleV1 | RuleV2
): rule is PubkeyMatchRuleV1 =>
  isRuleV1(rule) && 'PubkeyMatch' in (rule as object);
