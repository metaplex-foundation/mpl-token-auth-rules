import * as beet from '@metaplex-foundation/beet';
import { deserializeRulesV2, RuleV2, serializeRuleHeaderV2, serializeRulesV2 } from './rule';
import { RuleTypeV2 } from './ruleType';

export type AnyRuleV2 = {
  type: RuleTypeV2.Any;
  rules: RuleV2[];
};

export const anyV2 = (rules: RuleV2[]): AnyRuleV2 => ({
  type: RuleTypeV2.Any,
  rules,
});

export const serializeAnyV2 = (anyRule: AnyRuleV2): Buffer => {
  const sizeBuffer = Buffer.alloc(8);
  beet.u64.write(sizeBuffer, 0, anyRule.rules.length);
  const rulesBuffer = serializeRulesV2(anyRule.rules);
  const headerBuffer = serializeRuleHeaderV2(RuleTypeV2.Any, rulesBuffer.length);
  return Buffer.concat([headerBuffer, sizeBuffer, rulesBuffer]);
};

export const deserializeAnyV2 = (buffer: Buffer, offset = 0): AnyRuleV2 => {
  offset += 8;
  const size: beet.bignum = beet.u64.read(buffer, offset);
  offset += 8;
  const rules = deserializeRulesV2(buffer, size, offset);
  return { type: RuleTypeV2.Any, rules };
};
