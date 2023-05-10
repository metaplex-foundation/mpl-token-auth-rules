import { RuleSetRevisionV1, deserializeRuleSetV1, serializeRuleSetV1 } from './ruleSetV1';
import { RuleSetV2, deserializeRuleSetV2, serializeRuleSetV2 } from './ruleSetV2';

export type RuleSetRevision = RuleSetRevisionV1 | RuleSetV2;

export const isRuleSetV1 = (ruleSet: RuleSetRevision): ruleSet is RuleSetRevisionV1 => {
  return (ruleSet as RuleSetRevisionV1).libVersion === 1;
};

export const isRuleSetV2 = (ruleSet: RuleSetRevision): ruleSet is RuleSetV2 => {
  return (ruleSet as RuleSetV2).libVersion === 2;
};

export const serializeRuleSetRevision = (ruleSet: RuleSetRevision): Buffer => {
  if (isRuleSetV1(ruleSet)) return serializeRuleSetV1(ruleSet);
  if (isRuleSetV2(ruleSet)) return serializeRuleSetV2(ruleSet);
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  throw new Error('Unknown rule set version: ' + (ruleSet as any).libVersion);
};

export const deserializeRuleSetRevision = (buffer: Buffer, offset = 0): RuleSetRevision => {
  const libVersion = buffer[offset];
  if (libVersion === 1) return deserializeRuleSetV1(buffer, offset);
  if (libVersion === 2) return deserializeRuleSetV2(buffer, offset);
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  throw new Error('Unknown rule set version: ' + libVersion);
};

export const getRuleSetRevisionFromJson = (json: string): RuleSetRevision => {
  const ruleSet = JSON.parse(json);
  if (isRuleSetV1(ruleSet) || isRuleSetV2(ruleSet)) return ruleSet;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  throw new Error('Unknown rule set version: ' + (ruleSet as any).libVersion);
};
