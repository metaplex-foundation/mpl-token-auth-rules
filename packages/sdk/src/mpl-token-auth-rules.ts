import {
  RuleSetHeader,
  RuleSetRevisionMapV1,
  ruleSetHeaderBeet,
  ruleSetRevisionMapV1Beet,
} from './generated';
import {
  PublicKey
} from '@solana/web3.js'

import { decode } from '@msgpack/msgpack';

export * from './errors';
// @ts-ignore
export * from './generated';

export const PREFIX = 'rule_set';

export * from './pda';

export const getHeader = (data: Buffer): RuleSetHeader => {
  const [header, _] = ruleSetHeaderBeet.deserialize(data.slice(0, 9));
  return header;
};

export const getRevisionMapV1 = (data: Buffer): RuleSetRevisionMapV1 => {
  const header = getHeader(data);
  const [revmap, _] = ruleSetRevisionMapV1Beet.deserialize(
    data.slice(parseInt(header.revMapVersionLocation) + 1, data.length),
  );
  return revmap;
};

export const getLatestRuleSet = (data: Buffer): any => {
  const header = getHeader(data);
  const revmap = getRevisionMapV1(data);
  const latestRevision = parseInt(
    revmap.ruleSetRevisions[revmap.ruleSetRevisions.length - 1] as any
  );
  const rulesetDecoded = decode(
    data.slice(
      latestRevision + 1,
      parseInt(header.revMapVersionLocation as any)
    )
  );
  return JSON.stringify(rulesetDecoded, null, 2);
}

export type RuleSetV1 = {
  version: number;
  owner: PublicKey;
  name: string;
  ruleset: Record<string, string>;
};

export const getLatestRuleSetV1 = (data: Buffer): RuleSetV1 => {
  const header = getHeader(data);
  const revmap = getRevisionMapV1(data);
  const latestRevision = parseInt(
    revmap.ruleSetRevisions[revmap.ruleSetRevisions.length - 1] as any
  );
  const rulesetDecoded = decode(
    data.slice(
      latestRevision + 1,
      parseInt(header.revMapVersionLocation as any)
    )
  );
  return {
    version: rulesetDecoded[0] as number,
    owner: new PublicKey(rulesetDecoded[1]),
    name: rulesetDecoded[2] as string,
    ruleset: rulesetDecoded[3] as Record<string, string>,
  };
};
