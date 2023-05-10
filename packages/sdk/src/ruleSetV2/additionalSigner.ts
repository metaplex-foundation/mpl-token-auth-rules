import { PublicKey } from '@solana/web3.js';
import { Base58PublicKey, toBase58PublicKey } from './base58PublicKey';
import { deserializePublicKey, serializePublicKey } from './helpers';
import { serializeRuleHeaderV2 } from './rule';
import { RuleTypeV2 } from './ruleType';

export type AdditionalSignerRuleV2 = {
  type: 'AdditionalSigner';
  publicKey: Base58PublicKey;
};

export const additionalSignerV2 = (publicKey: PublicKey | string): AdditionalSignerRuleV2 => ({
  type: 'AdditionalSigner',
  publicKey: toBase58PublicKey(publicKey),
});

export const serializeAdditionalSignerV2 = (rule: AdditionalSignerRuleV2): Buffer => {
  return Buffer.concat([
    serializeRuleHeaderV2(RuleTypeV2.AdditionalSigner, 32),
    serializePublicKey(rule.publicKey),
  ]);
};

export const deserializeAdditionalSignerV2 = (
  buffer: Buffer,
  offset = 0,
): AdditionalSignerRuleV2 => {
  const publicKey = deserializePublicKey(buffer, offset + 8);
  return additionalSignerV2(publicKey);
};
