import {
  RuleSetHeader,
  RuleSetRevisionMapV1,
  ruleSetHeaderBeet,
  ruleSetRevisionMapV1Beet,
} from './generated';

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
  const latestRevision = parseInt(revmap.ruleSetRevisions[revmap.ruleSetRevisions.length - 1]);
  const rulesetDecoded = decode(
    data.slice(latestRevision + 1, parseInt(header.revMapVersionLocation)),
  );
  return JSON.stringify(rulesetDecoded, null, 2);
};
