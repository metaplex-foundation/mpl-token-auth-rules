import {
  RuleSetHeader,
  ruleSetHeaderBeet,
  RuleSetRevisionMapV1,
  ruleSetRevisionMapV1Beet,
} from './generated';

import type { bignum } from '@metaplex-foundation/beet';
import { BN } from 'bn.js';
import { deserializeRuleSetRevision, RuleSetRevision } from './ruleSetRevision';

export * from './errors';
export * from './generated';
export * from './pda';
export * from './ruleSetRevision';
export * from './ruleSetV1';
export * from './ruleSetV2';
export * from './shared';

export const PREFIX = 'rule_set';

export const getHeader = (data: Buffer): RuleSetHeader => {
  const [header] = ruleSetHeaderBeet.deserialize(data.subarray(0, 9));
  return header;
};

export const getRevisionMapV1 = (data: Buffer): RuleSetRevisionMapV1 => {
  const header = getHeader(data);
  const [revmap] = ruleSetRevisionMapV1Beet.deserialize(
    data.subarray(bignumToNumber(header.revMapVersionLocation) + 1, data.length),
  );
  return revmap;
};

export const getLatestRuleSet = (data: Buffer): RuleSetRevision => {
  const header = getHeader(data);
  const revmap = getRevisionMapV1(data);
  const latestRevision = bignumToNumber(
    revmap.ruleSetRevisions[revmap.ruleSetRevisions.length - 1],
  );
  const endOfRuleSet = bignumToNumber(header.revMapVersionLocation);
  return deserializeRuleSetRevision(data.subarray(latestRevision, endOfRuleSet));
};

function bignumToNumber(bignum: bignum): number {
  return new BN(bignum).toNumber();
}
