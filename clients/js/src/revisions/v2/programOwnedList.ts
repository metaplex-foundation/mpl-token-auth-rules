import {
  Context,
  PublicKeyBase58,
  PublicKeyInput,
  Serializer,
  base58,
  base58PublicKey,
} from '@metaplex-foundation/umi';
import { wrapSerializerInRuleHeaderV2 } from './ruleHeader';
import { RuleTypeV2 } from './ruleType';

export type ProgramOwnedListRuleV2 = {
  type: 'ProgramOwnedList';
  field: string;
  programs: PublicKeyBase58[];
};

export const programOwnedListV2 = (
  field: string,
  programs: PublicKeyInput[]
): ProgramOwnedListRuleV2 => ({
  type: 'ProgramOwnedList',
  field,
  programs: programs.map(base58PublicKey),
});

export const getProgramOwnedListRuleV2Serializer = (
  context: Pick<Context, 'serializer'>
): Serializer<ProgramOwnedListRuleV2> => {
  const s = context.serializer;
  return wrapSerializerInRuleHeaderV2(
    context,
    RuleTypeV2.ProgramOwnedList,
    s.struct([
      ['field', s.string({ size: 32 })],
      [
        'programs',
        s.array(s.string({ encoding: base58, size: 32 }), {
          size: 'remainder', // TODO: Ensure this works.
        }),
      ],
    ])
  );
};
