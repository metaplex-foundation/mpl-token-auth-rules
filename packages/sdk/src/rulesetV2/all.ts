import * as beet from '@metaplex-foundation/beet';
import { deserializeRuleV2, RuleV2, serializeRuleV2, serializeRuleHeaderV2 } from './rule';
import { RuleTypeV2 } from './ruleType';
import BN from 'bn.js';

export type AllRuleV2 = {
  type: RuleTypeV2.All;
  rules: RuleV2[];
};

export const allV2 = (rules: RuleV2[]): AllRuleV2 => ({
  type: RuleTypeV2.All,
  rules,
});

export const serializeAllV2 = (allRule: AllRuleV2): Buffer => {
  const sizeBuffer = Buffer.alloc(8);
  beet.u64.write(sizeBuffer, 8, allRule.rules.length);
  const rulesBuffer = Buffer.concat(allRule.rules.map(serializeRuleV2));
  const headerBuffer = serializeRuleHeaderV2(RuleTypeV2.All, rulesBuffer.length);
  return Buffer.concat([headerBuffer, sizeBuffer, rulesBuffer]);
};

export const deserializeAllV2 = (buffer: Buffer, offset = 0): AllRuleV2 => {
  offset += 8;
  const size: beet.bignum = beet.u64.read(buffer, offset);
  offset += 8;
  const sizeAsNumber = new BN(size).toNumber();
  const rules: RuleV2[] = [];

  for (let index = 0; index < sizeAsNumber; index++) {
    const length = beet.u32.read(buffer, offset + 4);
    rules.push(deserializeRuleV2(buffer, offset));
    offset += 8 + length;
  }

  return { type: RuleTypeV2.All, rules };
};
