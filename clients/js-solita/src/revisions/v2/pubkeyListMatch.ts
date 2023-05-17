import * as beet from '@metaplex-foundation/beet';
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

export type PubkeyListMatchRuleV2 = {
  type: 'PubkeyListMatch';
  field: string;
  publicKeys: Base58PublicKey[];
};

export const pubkeyListMatchV2 = (
  field: string,
  publicKeys: (PublicKey | Base58PublicKey)[],
): PubkeyListMatchRuleV2 => ({
  type: 'PubkeyListMatch',
  field,
  publicKeys: publicKeys.map(toBase58PublicKey),
});

export const serializePubkeyListMatchV2 = (rule: PubkeyListMatchRuleV2): Buffer => {
  const length = 32 + 32 * rule.publicKeys.length;
  const headerBuffer = serializeRuleHeaderV2(RuleTypeV2.PubkeyListMatch, length);
  const fieldBuffer = serializeString32(rule.field);
  const publicKeyBuffers = rule.publicKeys.map((publicKey) => serializePublicKey(publicKey));
  return Buffer.concat([headerBuffer, fieldBuffer, ...publicKeyBuffers]);
};

export const deserializePubkeyListMatchV2 = (buffer: Buffer, offset = 0): PubkeyListMatchRuleV2 => {
  // Header.
  const length = beet.u32.read(buffer, offset + 4);
  const numberOfPublicKeys = Math.floor((length - 32) / 32);
  offset += 8;

  // Field.
  const field = deserializeString32(buffer, offset);
  offset += 32;

  // PublicKeys.
  const publicKeys = [];
  for (let index = 0; index < numberOfPublicKeys; index++) {
    publicKeys.push(deserializePublicKey(buffer, offset));
    offset += 32;
  }

  return pubkeyListMatchV2(field, publicKeys);
};
