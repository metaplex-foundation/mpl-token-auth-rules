import { BN } from 'bn.js';
import { getHeader, getRevisionMapV1 } from './revisionMap';
import { RuleSetRevisionV1, deserializeRuleSetRevisionV1, serializeRuleSetRevisionV1 } from './v1';
import { RuleSetRevisionV2, deserializeRuleSetRevisionV2, serializeRuleSetRevisionV2 } from './v2';

export type RuleSetRevision = RuleSetRevisionV1 | RuleSetRevisionV2;

export const isRuleSetV1 = (ruleSet: RuleSetRevision): ruleSet is RuleSetRevisionV1 => {
  return (ruleSet as RuleSetRevisionV1).libVersion === 1;
};

export const isRuleSetV2 = (ruleSet: RuleSetRevision): ruleSet is RuleSetRevisionV2 => {
  return (ruleSet as RuleSetRevisionV2).libVersion === 2;
};

export const serializeRuleSetRevision = (ruleSet: RuleSetRevision): Buffer => {
  if (isRuleSetV1(ruleSet)) return serializeRuleSetRevisionV1(ruleSet);
  if (isRuleSetV2(ruleSet)) return serializeRuleSetRevisionV2(ruleSet);
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  throw new Error('Unknown rule set version: ' + (ruleSet as any).libVersion);
};

export const deserializeRuleSetRevision = (buffer: Buffer, offset = 0): RuleSetRevision => {
  const libVersion = buffer[offset];
  if (libVersion === 1) return deserializeRuleSetRevisionV1(buffer, offset);
  if (libVersion === 2) return deserializeRuleSetRevisionV2(buffer, offset);
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  throw new Error('Unknown rule set version: ' + libVersion);
};

export const getRuleSetRevisionFromJson = (json: string): RuleSetRevision => {
  const ruleSet = JSON.parse(json);
  if (isRuleSetV1(ruleSet) || isRuleSetV2(ruleSet)) return ruleSet;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  throw new Error('Unknown rule set version: ' + (ruleSet as any).libVersion);
};

export const getLatestRuleSet = (data: Buffer): RuleSetRevision => {
  const header = getHeader(data);
  const revmap = getRevisionMapV1(data);
  const latestRevision = new BN(
    revmap.ruleSetRevisions[revmap.ruleSetRevisions.length - 1],
  ).toNumber();
  const endOfRuleSet = new BN(header.revMapVersionLocation).toNumber();
  return deserializeRuleSetRevision(data.subarray(latestRevision, endOfRuleSet));
};
