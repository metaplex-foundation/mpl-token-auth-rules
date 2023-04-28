import { serializeRuleHeaderV2 } from './rule';
import { RuleTypeV2 } from './ruleType';

export type PassRuleV2 = { type: RuleTypeV2.Pass };

export const passV2 = (): PassRuleV2 => ({ type: RuleTypeV2.Pass });

// eslint-disable-next-line @typescript-eslint/no-unused-vars
export const serializePassV2 = (rule: PassRuleV2): Buffer => {
  return serializeRuleHeaderV2(RuleTypeV2.Pass, 0);
};

// eslint-disable-next-line @typescript-eslint/no-unused-vars
export const deserializePassV2 = (buffer: Buffer, offset = 0): PassRuleV2 => {
  return { type: RuleTypeV2.Pass };
};
