import { Serializer, mergeBytes } from '@metaplex-foundation/umi/serializers';
import {
  getRuleSetHeaderSerializer,
  getRuleSetRevisionMapV1Serializer,
} from '../generated';

export type RuleSetRevisionMap = {
  version: 1;
  location: number;
  revisionLocations: number[];
};

export const getRuleSetRevisionMapSerializer = (
  location: number
): Serializer<RuleSetRevisionMap> => ({
  description: 'RuleSetRevisionMap',
  fixedSize: null,
  maxSize: null,
  serialize: (revisionMap) => {
    if (revisionMap.version !== 1) {
      throw new Error(
        `Unsupported revision map version: ${revisionMap.version}`
      );
    }
    const revisionMapV1 = getRuleSetRevisionMapV1Serializer().serialize({
      ruleSetRevisions: revisionMap.revisionLocations.map((n) => BigInt(n)),
    });
    return mergeBytes([new Uint8Array([revisionMap.version]), revisionMapV1]);
  },
  deserialize: (buffer, offset = 0) => {
    const version = Number(buffer[offset]);
    if (version !== 1) {
      throw new Error(`Unsupported revision map version: ${version}`);
    }
    const [revisionMapV1, newOffset] =
      getRuleSetRevisionMapV1Serializer().deserialize(buffer, offset + 1);
    const revisionLocations = revisionMapV1.ruleSetRevisions.map((n) =>
      Number(n)
    );
    return [{ version, location, revisionLocations }, newOffset];
  },
});

export const getRuleSetRevisionMapFromAccountData = (
  accountData: Uint8Array
): RuleSetRevisionMap => {
  const [header] = getRuleSetHeaderSerializer().deserialize(accountData);
  const location = Number(header.revMapVersionLocation);
  const [revisionMap] = getRuleSetRevisionMapSerializer(location).deserialize(
    accountData,
    location
  );
  return revisionMap;
};
