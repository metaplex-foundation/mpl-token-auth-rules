import { encode, decode } from '@msgpack/msgpack';
import type { PublicKeyAsArrayOfBytes } from './publicKey';
import type { RuleV1 } from './rule';

export type RuleSetV1 = {
  /** The version of the ruleset. */
  libVersion: 1;
  /** The name of the ruleset. */
  ruleSetName: string;
  /** The owner of the ruleset as an array of 32 bytes. */
  owner: PublicKeyAsArrayOfBytes;
  /** The operations of the ruleset. */
  operations: Record<string, RuleV1>;
};

export const serializeRuleSetV1 = (ruleSet: RuleSetV1): Buffer => {
  return Buffer.from(encode(ruleSet));
};

export const deserializeRuleSetV1 = (buffer: Buffer, offset = 0): RuleSetV1 => {
  const ruleSet = decode(buffer.subarray(offset + 1));

  if (Array.isArray(ruleSet)) {
    return {
      libVersion: ruleSet[0],
      owner: ruleSet[1],
      ruleSetName: ruleSet[2],
      operations: ruleSet[3],
    };
  }

  return ruleSet as RuleSetV1;
};
