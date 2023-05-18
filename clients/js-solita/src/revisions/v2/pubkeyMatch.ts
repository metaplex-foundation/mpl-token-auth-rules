import { PublicKey } from '@solana/web3.js';
import {
  deserializePublicKey,
  deserializeString32,
  serializePublicKey,
  serializeString32,
} from './helpers';
import { serializeRuleHeaderV2 } from './rule';
import { RuleTypeV2 } from './ruleType';
import { Base58PublicKey, toBase58PublicKey } from './base58PublicKey';

export type PubkeyMatchRuleV2 = {
  type: 'PubkeyMatch';
  field: string;
  publicKey: Base58PublicKey;
};

export const pubkeyMatchV2 = (
  field: string,
  publicKey: PublicKey | Base58PublicKey,
): PubkeyMatchRuleV2 => ({
  type: 'PubkeyMatch',
  publicKey: toBase58PublicKey(publicKey),
  field,
});

export const serializePubkeyMatchV2 = (rule: PubkeyMatchRuleV2): Buffer => {
  return Buffer.concat([
    serializeRuleHeaderV2(RuleTypeV2.PubkeyMatch, 64),
    serializePublicKey(rule.publicKey),
    serializeString32(rule.field),
  ]);
};

export const deserializePubkeyMatchV2 = (buffer: Buffer, offset = 0): PubkeyMatchRuleV2 => {
  offset += 8; // Skip rule header.
  const publicKey = deserializePublicKey(buffer, offset);
  offset += 32;
  const field = deserializeString32(buffer, offset);
  offset += 32;

  return pubkeyMatchV2(field, publicKey);
};
