import { Option, isNone, none, some } from '@metaplex-foundation/umi';
import {
  Serializer,
  bytes,
  mapSerializer,
  u32,
} from '@metaplex-foundation/umi/serializers';
import { RuleSetRevision, getRuleSetRevisionSerializer } from '../revisions';

export type RuleSetRevisionInput = Option<RuleSetRevision>;

export type RuleSetRevisionInputArgs = RuleSetRevisionInput;

export const getRuleSetRevisionInputSerializer = (): Serializer<
  RuleSetRevisionInputArgs,
  RuleSetRevisionInput
> => {
  const ruleSetSerializer = getRuleSetRevisionSerializer();
  return mapSerializer(
    bytes({ size: u32() }),
    (revision: RuleSetRevisionInputArgs): Uint8Array => {
      if (isNone(revision)) return new Uint8Array();
      return ruleSetSerializer.serialize(revision.value);
    },
    (buffer: Uint8Array): RuleSetRevisionInput => {
      if (buffer.length === 0) return none();
      const [ruleSet] = ruleSetSerializer.deserialize(buffer, 4);
      return some(ruleSet);
    }
  );
};
