import * as beet from '@metaplex-foundation/beet';
import { deserializeRule, Rule, serializeRule, serializeRuleHeader } from './rule';
import { RuleType } from './ruleType';
import BN from 'bn.js';

export type AllRule = {
  type: RuleType.All;
  rules: Rule[];
};

export const all = (rules: Rule[]): AllRule => ({
  type: RuleType.All,
  rules,
});

export const serializeAll = (allRule: AllRule): Buffer => {
  const sizeBuffer = Buffer.alloc(8);
  beet.u64.write(sizeBuffer, 8, allRule.rules.length);
  const rulesBuffer = Buffer.concat(allRule.rules.map(serializeRule));
  const headerBuffer = serializeRuleHeader(RuleType.All, rulesBuffer.length);
  return Buffer.concat([headerBuffer, sizeBuffer, rulesBuffer]);
};

export const deserializeAll = (buffer: Buffer, offset = 0): AllRule => {
  offset += 8;
  const size: beet.bignum = beet.u64.read(buffer, offset);
  offset += 8;
  const sizeAsNumber = new BN(size).toNumber();
  const rules: Rule[] = [];

  for (let index = 0; index < sizeAsNumber; index++) {
    const length = beet.u32.read(buffer, offset + 4);
    rules.push(deserializeRule(buffer, offset));
    offset += 8 + length;
  }

  return { type: RuleType.All, rules };
};
