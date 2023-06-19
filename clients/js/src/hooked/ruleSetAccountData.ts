import { Serializer } from '@metaplex-foundation/umi/serializers';
import { Key, getRuleSetHeaderSerializer } from '../generated';
import {
  RuleSetRevision,
  RuleSetRevisionMap,
  getRuleSetRevisionMapSerializer,
  getRuleSetRevisionSerializer,
} from '../revisions';

export type RuleSetAccountData = {
  key: Key;
  latestRevision: RuleSetRevision;
  revisions: RuleSetRevision[];
  revisionMap: RuleSetRevisionMap;
};

export type RuleSetAccountDataArgs = RuleSetAccountData;

export const getRuleSetAccountDataSerializer = (): Serializer<
  RuleSetAccountDataArgs,
  RuleSetAccountData
> => ({
  description: 'RuleSetAccountData',
  fixedSize: null,
  maxSize: null,
  serialize: () => {
    throw new Error('Operation not supported.');
  },
  deserialize: (
    buffer: Uint8Array,
    offset = 0
  ): [RuleSetAccountData, number] => {
    // Header and revision map.
    const [header] = getRuleSetHeaderSerializer().deserialize(buffer, offset);
    if (header.key !== Key.RuleSet) {
      throw new Error(
        `Expected a RuleSet account, got account data key: ${header.key}`
      );
    }
    const revisionMapLocation = Number(header.revMapVersionLocation);
    const [revisionMap, finalOffset] = getRuleSetRevisionMapSerializer(
      revisionMapLocation
    ).deserialize(buffer, offset + revisionMapLocation);

    // Revisions.
    const revisions = revisionMap.revisionLocations.map((location, index) => {
      const revisionStart = offset + location;
      const revisionEnd =
        offset +
        (revisionMap.revisionLocations[index + 1] ?? revisionMapLocation);
      const revisionSlice = buffer.slice(revisionStart, revisionEnd);
      return getRuleSetRevisionSerializer().deserialize(revisionSlice)[0];
    });

    return [
      {
        key: Key.RuleSet,
        latestRevision: revisions[revisions.length - 1],
        revisions,
        revisionMap,
      },
      finalOffset,
    ];
  },
});
