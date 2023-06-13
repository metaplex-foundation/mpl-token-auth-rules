import {
  Context,
  PublicKey,
  PublicKeyInput,
  Serializer,
  publicKey as toPublicKey,
} from '@metaplex-foundation/umi';
import type { RuleV1 } from '../v1';
import { RuleV2, isRuleV2 } from './rule';
import { wrapSerializerInRuleHeaderV2 } from './ruleHeader';
import { RuleTypeV2 } from './ruleType';

export type PdaMatchRuleV2 = {
  type: 'PdaMatch';
  pdaField: string;
  program: PublicKey;
  seedsField: string;
};

export const pdaMatchV2 = (
  pdaField: string,
  program: PublicKeyInput,
  seedsField: string
): PdaMatchRuleV2 => ({
  type: 'PdaMatch',
  pdaField,
  program: toPublicKey(program),
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
      ['program', s.publicKey()],
      ['pdaField', s.string({ size: 32 })],
      ['seedsField', s.string({ size: 32 })],
    ])
  );
};

export const isPdaMatchRuleV2 = (
  rule: RuleV1 | RuleV2
): rule is PdaMatchRuleV2 => isRuleV2(rule) && rule.type === 'PdaMatch';
