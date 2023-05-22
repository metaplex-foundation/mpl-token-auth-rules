import { RuleV2 } from "../v2";
import { RuleV1, isRuleV1 } from "./rule";

export type PubkeyTreeMatchRuleV1 = {
  PubkeyTreeMatch: {
    root: number[];
    pubkey_field: string;
    proof_field: string;
  };
};

export const isPubkeyTreeMatchRuleV1 = (
  rule: RuleV1 | RuleV2
): rule is PubkeyTreeMatchRuleV1 =>
  isRuleV1(rule) && 'PubkeyTreeMatch' in (rule as object);
