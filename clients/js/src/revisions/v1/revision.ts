import { Serializer } from '@metaplex-foundation/umi';
import { decode, encode } from '@msgpack/msgpack';
import type { PublicKeyAsArrayOfBytes } from './publicKey';
import type { RuleV1 } from './rule';

export type RuleSetRevisionV1 = {
  /** The version of the ruleset. */
  libVersion: 1;
  /** The name of the ruleset. */
  ruleSetName: string;
  /** The owner of the ruleset as an array of 32 bytes. */
  owner: PublicKeyAsArrayOfBytes;
  /** The operations of the ruleset. */
  operations: Record<string, RuleV1>;
};

export const getRuleSetRevisionV1Serializer =
  (): Serializer<RuleSetRevisionV1> => ({
    description: 'RuleSetRevisionV1',
    fixedSize: null,
    maxSize: null,
    serialize: (revision: RuleSetRevisionV1): Uint8Array => encode(revision),
    deserialize: (
      buffer: Uint8Array,
      offset = 0
    ): [RuleSetRevisionV1, number] => {
      const ruleSet = decode(buffer.subarray(offset + 1));
      const newOffset = offset + buffer.length;

      if (Array.isArray(ruleSet)) {
        return [
          {
            libVersion: ruleSet[0],
            owner: ruleSet[1],
            ruleSetName: ruleSet[2],
            operations: ruleSet[3],
          },
          newOffset,
        ];
      }

      return [ruleSet as RuleSetRevisionV1, newOffset];
    },
  });
