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

export type ProgramOwnedRuleV2 = {
  type: 'ProgramOwned';
  field: string;
  program: PublicKeyBase58;
};

export const programOwnedV2 = (
  field: string,
  program: PublicKeyInput
): ProgramOwnedRuleV2 => ({
  type: 'ProgramOwned',
  program: base58PublicKey(program),
  field,
});

export const getProgramOwnedRuleV2Serializer = (
  context: Pick<Context, 'serializer'>
): Serializer<ProgramOwnedRuleV2> => {
  const s = context.serializer;
  return wrapSerializerInRuleHeaderV2(
    context,
    RuleTypeV2.ProgramOwned,
    s.struct([
      ['program', s.string({ encoding: base58, size: 32 })],
      ['field', s.string({ size: 32 })],
    ])
  );
};
