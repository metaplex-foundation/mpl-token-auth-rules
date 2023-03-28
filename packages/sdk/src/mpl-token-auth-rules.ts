import {
  RuleSetHeader,
  ruleSetHeaderBeet,
  RuleSetRevisionMapV1,
  ruleSetRevisionMapV1Beet,
} from './generated';

import type { bignum } from '@metaplex-foundation/beet';
import { decode } from '@msgpack/msgpack';
import { BN } from 'bn.js';

export * from './errors';
export * from './generated';
export * from './pda';
export * from './rulesetV2';

export const PREFIX = 'rule_set';

export const getHeader = (data: Buffer): RuleSetHeader => {
  const [header, _] = ruleSetHeaderBeet.deserialize(data.slice(0, 9));
  return header;
};

export const getRevisionMapV1 = (data: Buffer): RuleSetRevisionMapV1 => {
  const header = getHeader(data);
  const [revmap] = ruleSetRevisionMapV1Beet.deserialize(
    data.slice(bignumToNumber(header.revMapVersionLocation) + 1, data.length),
  );
  return revmap;
};

export const getLatestRuleSet = (data: Buffer): any => {
  const header = getHeader(data);
  const revmap = getRevisionMapV1(data);
  const latestRevision = bignumToNumber(
    revmap.ruleSetRevisions[revmap.ruleSetRevisions.length - 1],
  );
  const rulesetDecoded = decode(
    data.slice(latestRevision + 1, bignumToNumber(header.revMapVersionLocation)),
  );
  return JSON.stringify(rulesetDecoded, null, 2);
};

function bignumToNumber(bignum: bignum): number {
  return new BN(bignum).toNumber();
}
