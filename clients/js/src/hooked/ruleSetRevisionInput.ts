import {
  Context,
  Option,
  Serializer,
  isNone,
  mapSerializer,
  none,
  some,
} from '@metaplex-foundation/umi';
import { RuleSetRevision, getRuleSetRevisionSerializer } from '../revisions';

export type RuleSetRevisionInput = Option<RuleSetRevision>;

export type RuleSetRevisionInputArgs = RuleSetRevisionInput;

export const getRuleSetRevisionInputSerializer = (
  context: Pick<Context, 'serializer'>
): Serializer<RuleSetRevisionInputArgs, RuleSetRevisionInput> => {
  const ruleSetSerializer = getRuleSetRevisionSerializer(context);
  const s = context.serializer;
  return mapSerializer(
    s.bytes({ size: s.u32() }),
    (revision: RuleSetRevisionInputArgs): Uint8Array => {
      if (isNone(revision)) return new Uint8Array();
      return ruleSetSerializer.serialize(revision.value);
    },
    (bytes: Uint8Array): RuleSetRevisionInput => {
      if (bytes.length === 0) return none();
      const [ruleSet] = ruleSetSerializer.deserialize(bytes, 4);
      return some(ruleSet);
    }
  );
};
