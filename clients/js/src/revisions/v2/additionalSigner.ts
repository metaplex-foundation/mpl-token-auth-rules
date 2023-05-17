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

export type AdditionalSignerRuleV2 = {
  type: 'AdditionalSigner';
  publicKey: PublicKeyBase58;
};

export const additionalSignerV2 = (
  publicKey: PublicKeyInput
): AdditionalSignerRuleV2 => ({
  type: 'AdditionalSigner',
  publicKey: base58PublicKey(publicKey),
});

export const getAdditionalSignerRuleV2Serializer = (
  context: Pick<Context, 'serializer'>
): Serializer<AdditionalSignerRuleV2> => {
  const s = context.serializer;
  return wrapSerializerInRuleHeaderV2(
    context,
    RuleTypeV2.AdditionalSigner,
    s.struct([['publicKey', s.string({ encoding: base58, size: 32 })]])
  );
};
