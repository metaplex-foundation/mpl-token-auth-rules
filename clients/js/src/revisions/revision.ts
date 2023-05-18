import { Context, Serializer } from '@metaplex-foundation/umi';
import { getRuleSetRevisionMapFromAccountData } from './revisionMap';
import { RuleSetRevisionV1, getRuleSetRevisionV1Serializer } from './v1';
import { RuleSetRevisionV2, getRuleSetRevisionV2Serializer } from './v2';

export type RuleSetRevision = RuleSetRevisionV1 | RuleSetRevisionV2;

export const getRuleSetRevisionSerializer = (
  context: Pick<Context, 'serializer'>
): Serializer<RuleSetRevision> => ({
  description: 'RuleSetRevision',
  fixedSize: null,
  maxSize: null,
  serialize: (revision: RuleSetRevision) =>
    getRuleSetRevisionSerializerFromVersion(
      context,
      revision.libVersion
    ).serialize(revision),
  deserialize: (buffer, offset = 0) =>
    getRuleSetRevisionSerializerFromVersion(
      context,
      buffer[offset] as RuleSetRevision['libVersion']
    ).deserialize(buffer, offset),
});

export const getRuleSetRevisionSerializerFromVersion = <
  T extends RuleSetRevision
>(
  context: Pick<Context, 'serializer'>,
  version: T['libVersion']
): Serializer<T> =>
  ((): Serializer<any> => {
    switch (version) {
      case 1:
        return getRuleSetRevisionV1Serializer();
      case 2:
        return getRuleSetRevisionV2Serializer(context);
      default:
        throw new Error(`Unknown rule set revision version: ${version}`);
    }
  })() as Serializer<T>;

export const isRuleSetV1 = (
  ruleSet: RuleSetRevision
): ruleSet is RuleSetRevisionV1 =>
  (ruleSet as RuleSetRevisionV1).libVersion === 1;

export const isRuleSetV2 = (
  ruleSet: RuleSetRevision
): ruleSet is RuleSetRevisionV2 =>
  (ruleSet as RuleSetRevisionV2).libVersion === 2;

export const getRuleSetRevisionFromJson = (json: string): RuleSetRevision => {
  const ruleSet = JSON.parse(json);
  if (isRuleSetV1(ruleSet) || isRuleSetV2(ruleSet)) return ruleSet;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  throw new Error(`Unknown rule set version: ${(ruleSet as any).libVersion}`);
};

export const getLatestRuleSetRevision = (
  context: Pick<Context, 'serializer'>,
  buffer: Uint8Array
): RuleSetRevision => {
  const revisionMap = getRuleSetRevisionMapFromAccountData(context, buffer);
  const latestRevisionStart =
    revisionMap.revisionLocations[revisionMap.revisionLocations.length - 1];
  const latestRevisionEnd = revisionMap.location;
  return getRuleSetRevisionSerializer(context).deserialize(
    buffer.slice(latestRevisionStart, latestRevisionEnd)
  )[0];
};
