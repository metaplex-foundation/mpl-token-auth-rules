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

export type PdaMatchRuleV2 = {
  type: 'PdaMatch';
  pdaField: string;
  program: PublicKeyBase58;
  seedsField: string;
};

export const pdaMatchV2 = (
  pdaField: string,
  program: PublicKeyInput,
  seedsField: string
): PdaMatchRuleV2 => ({
  type: 'PdaMatch',
  pdaField,
  program: base58PublicKey(program),
  seedsField,
});

export const getPdaMatchRuleV2Serializer = (
  context: Pick<Context, 'serializer'>
): Serializer<PdaMatchRuleV2> => {
  const s = context.serializer;
  return wrapSerializerInRuleHeaderV2(
    context,
    RuleTypeV2.PdaMatch,
    s.struct([
      ['program', s.string({ encoding: base58, size: 32 })],
      ['pdaField', s.string({ size: 32 })],
      ['seedsField', s.string({ size: 32 })],
    ])
  );
};
