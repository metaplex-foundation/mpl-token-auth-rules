import { BN } from 'bn.js';
import {
  RuleSetHeader,
  RuleSetRevisionMapV1,
  ruleSetHeaderBeet,
  ruleSetRevisionMapV1Beet,
} from '../generated';

export const getHeader = (data: Buffer): RuleSetHeader => {
  const [header] = ruleSetHeaderBeet.deserialize(data.subarray(0, 9));
  return header;
};

export const getRevisionMapV1 = (data: Buffer): RuleSetRevisionMapV1 => {
  const header = getHeader(data);
  const revisionMapLocation = new BN(header.revMapVersionLocation).toNumber();
  const [revisionMap] = ruleSetRevisionMapV1Beet.deserialize(
    data.subarray(revisionMapLocation + 1, data.length),
  );
  return revisionMap;
};
