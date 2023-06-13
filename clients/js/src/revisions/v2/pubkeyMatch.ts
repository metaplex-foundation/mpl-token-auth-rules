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

export type PubkeyMatchRuleV2 = {
  type: 'PubkeyMatch';
  field: string;
  publicKey: PublicKey;
};

export const pubkeyMatchV2 = (
  field: string,
  publicKey: PublicKeyInput
): PubkeyMatchRuleV2 => ({
  type: 'PubkeyMatch',
  publicKey: toPublicKey(publicKey),
  field,
});

export const getPubkeyMatchRuleV2Serializer = (
  context: Pick<Context, 'serializer'>
): Serializer<PubkeyMatchRuleV2> => {
  const s = context.serializer;
  return wrapSerializerInRuleHeaderV2(
    context,
    RuleTypeV2.PubkeyMatch,
    s.struct([
      ['publicKey', s.publicKey()],
      ['field', s.string({ size: 32 })],
    ])
  );
};

export const isPubkeyMatchRuleV2 = (
  rule: RuleV1 | RuleV2
): rule is PubkeyMatchRuleV2 => isRuleV2(rule) && rule.type === 'PubkeyMatch';
