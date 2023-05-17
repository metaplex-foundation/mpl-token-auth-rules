import { Context, Serializer, mergeBytes } from '@metaplex-foundation/umi';
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
  context: Pick<Context, 'serializer'>,
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
    const revisionMapV1 = getRuleSetRevisionMapV1Serializer(context).serialize({
      ruleSetRevisions: revisionMap.revisionLocations.map((n) => BigInt(n)),
    });
    return mergeBytes([new Uint8Array([revisionMap.version]), revisionMapV1]);
  },
  deserialize: (buffer, offset = 0) => {
    const version = Number(buffer[offset]);
    if (version !== 1) {
      throw new Error(`Unsupported revision map version: ${version}`);
    }
    const [revisionMapV1, newOffset] = getRuleSetRevisionMapV1Serializer(
      context
    ).deserialize(buffer, offset + 1);
    const revisionLocations = revisionMapV1.ruleSetRevisions.map((n) =>
      Number(n)
    );
    return [{ version, location, revisionLocations }, newOffset];
  },
});

export const getRuleSetRevisionMapFromAccountData = (
  context: Pick<Context, 'serializer'>,
  accountData: Uint8Array
): RuleSetRevisionMap => {
  const [header] = getRuleSetHeaderSerializer(context).deserialize(accountData);
  const location = Number(header.revMapVersionLocation);
  const [revisionMap] = getRuleSetRevisionMapSerializer(
    context,
    location
  ).deserialize(accountData, location);
  return revisionMap;
};
