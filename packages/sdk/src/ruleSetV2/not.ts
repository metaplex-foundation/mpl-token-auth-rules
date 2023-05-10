import { deserializeRuleV2, RuleV2, serializeRuleHeaderV2, serializeRuleV2 } from './rule';
import { RuleTypeV2 } from './ruleType';

export type NotRuleV2 = {
  type: 'Not';
  rule: RuleV2;
};

export const notV2 = (rule: RuleV2): NotRuleV2 => ({ type: 'Not', rule });

export const serializeNotV2 = (notRule: NotRuleV2): Buffer => {
  const ruleBuffer = serializeRuleV2(notRule.rule);
  const headerBuffer = serializeRuleHeaderV2(RuleTypeV2.Not, ruleBuffer.length);
  return Buffer.concat([headerBuffer, ruleBuffer]);
};

export const deserializeNotV2 = (buffer: Buffer, offset = 0): NotRuleV2 => {
  const rule = deserializeRuleV2(buffer, offset + 8);
  return notV2(rule);
};
