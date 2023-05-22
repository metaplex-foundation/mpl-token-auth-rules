import { RuleV2 } from '../v2';
import { RuleV1, isRuleV1 } from './rule';

export type ProgramOwnedTreeRuleV1 = {
  ProgramOwnedTree: {
    root: number[];
    pubkey_field: string;
    proof_field: string;
  };
};

export const isProgramOwnedTreeRuleV1 = (
  rule: RuleV1 | RuleV2
): rule is ProgramOwnedTreeRuleV1 =>
  isRuleV1(rule) && 'ProgramOwnedTree' in (rule as object);
