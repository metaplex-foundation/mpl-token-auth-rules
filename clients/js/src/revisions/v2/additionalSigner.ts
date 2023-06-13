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

export type AdditionalSignerRuleV2 = {
  type: 'AdditionalSigner';
  publicKey: PublicKey;
};

export const additionalSignerV2 = (
  publicKey: PublicKeyInput
): AdditionalSignerRuleV2 => ({
  type: 'AdditionalSigner',
  publicKey: toPublicKey(publicKey),
});

export const getAdditionalSignerRuleV2Serializer = (
  context: Pick<Context, 'serializer'>
): Serializer<AdditionalSignerRuleV2> => {
  const s = context.serializer;
  return wrapSerializerInRuleHeaderV2(
    context,
    RuleTypeV2.AdditionalSigner,
    s.struct([['publicKey', s.publicKey()]])
  );
};

export const isAdditionalSignerRuleV2 = (
  rule: RuleV1 | RuleV2
): rule is AdditionalSignerRuleV2 =>
  isRuleV2(rule) && rule.type === 'AdditionalSigner';
