import * as beetSolana from '@metaplex-foundation/beet-solana';
import { PublicKey } from '@solana/web3.js';
import { serializeRuleHeaderV2 } from './rule';
import { RuleTypeV2 } from './ruleType';

export type AdditionalSignerRuleV2 = {
  type: RuleTypeV2.AdditionalSigner;
  publicKey: PublicKey;
};

export const additionalSignerV2 = (publicKey: PublicKey): AdditionalSignerRuleV2 => ({
  type: RuleTypeV2.AdditionalSigner,
  publicKey,
});

export const serializeAdditionalSignerV2 = (rule: AdditionalSignerRuleV2): Buffer => {
  const headerBuffer = serializeRuleHeaderV2(RuleTypeV2.AdditionalSigner, 32);
  const buffer = Buffer.alloc(32);
  beetSolana.publicKey.write(buffer, 0, rule.publicKey);
  return Buffer.concat([headerBuffer, buffer]);
};

export const deserializeAdditionalSignerV2 = (
  buffer: Buffer,
  offset = 0,
): AdditionalSignerRuleV2 => {
  const publicKey = beetSolana.publicKey.read(buffer, offset + 8);
  return { type: RuleTypeV2.AdditionalSigner, publicKey };
};
