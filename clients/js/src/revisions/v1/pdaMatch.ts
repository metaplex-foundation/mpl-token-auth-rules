import { RuleV2 } from "../v2";
import { RuleV1, isRuleV1 } from "./rule";
import { PublicKeyAsArrayOfBytes } from './publicKey';

export type PdaMatchRuleV1 = {
  PDAMatch: {
    program: PublicKeyAsArrayOfBytes;
    pda_field: string;
    seeds_field: string;
  };
};

export const isPdaMatchRuleV1 = (
  rule: RuleV1 | RuleV2
): rule is PdaMatchRuleV1 => isRuleV1(rule) && 'PDAMatch' in (rule as object);
