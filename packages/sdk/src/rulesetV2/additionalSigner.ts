import * as beetSolana from '@metaplex-foundation/beet-solana';
import { PublicKey } from '@solana/web3.js';
import { serializeRuleHeader } from './rule';
import { RuleType } from './ruleType';

export type AdditionalSignerRule = {
  type: RuleType.AdditionalSigner;
  publicKey: PublicKey;
};

export const additionalSigner = (publicKey: PublicKey): AdditionalSignerRule => ({
  type: RuleType.AdditionalSigner,
  publicKey,
});

export const serializeAdditionalSigner = (rule: AdditionalSignerRule): Buffer => {
  const headerBuffer = serializeRuleHeader(RuleType.AdditionalSigner, 32);
  const buffer = Buffer.alloc(32);
  beetSolana.publicKey.write(buffer, 8, rule.publicKey);
  return Buffer.concat([headerBuffer, buffer]);
};

export const deserializeAdditionalSigner = (buffer: Buffer, offset = 0): AdditionalSignerRule => {
  const publicKey = beetSolana.publicKey.read(buffer, offset + 8);
  return { type: RuleType.AdditionalSigner, publicKey };
};
