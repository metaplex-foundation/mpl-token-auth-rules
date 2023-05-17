import * as beet from '@metaplex-foundation/beet';
import { deserializeRulesV2, RuleV2, serializeRuleHeaderV2, serializeRulesV2 } from './rule';
import { RuleTypeV2 } from './ruleType';

export type AllRuleV2 = {
  type: 'All';
  rules: RuleV2[];
};

export const allV2 = (rules: RuleV2[]): AllRuleV2 => ({ type: 'All', rules });

export const serializeAllV2 = (allRule: AllRuleV2): Buffer => {
  const sizeBuffer = Buffer.alloc(8);
  beet.u64.write(sizeBuffer, 0, allRule.rules.length);
  const rulesBuffer = serializeRulesV2(allRule.rules);
  const headerBuffer = serializeRuleHeaderV2(
    RuleTypeV2.All,
    sizeBuffer.length + rulesBuffer.length,
  );
  return Buffer.concat([headerBuffer, sizeBuffer, rulesBuffer]);
};

export const deserializeAllV2 = (buffer: Buffer, offset = 0): AllRuleV2 => {
  offset += 8;
  const size: beet.bignum = beet.u64.read(buffer, offset);
  offset += 8;
  const rules = deserializeRulesV2(buffer, size, offset);
  return allV2(rules);
};
