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

export type ProgramOwnedRuleV2 = {
  type: 'ProgramOwned';
  field: string;
  program: PublicKey;
};

export const programOwnedV2 = (
  field: string,
  program: PublicKeyInput
): ProgramOwnedRuleV2 => ({
  type: 'ProgramOwned',
  program: toPublicKey(program),
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
      ['program', s.publicKey()],
      ['field', s.string({ size: 32 })],
    ])
  );
};

export const isProgramOwnedRuleV2 = (
  rule: RuleV1 | RuleV2
): rule is ProgramOwnedRuleV2 => isRuleV2(rule) && rule.type === 'ProgramOwned';
