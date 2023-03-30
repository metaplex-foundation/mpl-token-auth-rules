import * as beetSolana from '@metaplex-foundation/beet-solana';
import { PublicKey } from '@solana/web3.js';
import { serializeRuleHeaderV2 } from './rule';
import { RuleTypeV2 } from './ruleType';

export type PubkeyMatchRuleV2 = {
  type: RuleTypeV2.PubkeyMatch;
  publicKey: PublicKey;
  field: string;
};

export const pubkeyMatchV2 = (publicKey: PublicKey, field: string): PubkeyMatchRuleV2 => ({
  type: RuleTypeV2.PubkeyMatch,
  publicKey,
  field,
});

export const serializePubkeyMatchV2 = (rule: PubkeyMatchRuleV2): Buffer => {
  const headerBuffer = serializeRuleHeaderV2(RuleTypeV2.PubkeyMatch, 64);
  // PublicKey.
  const publicKeyBuffer = Buffer.alloc(32);
  beetSolana.publicKey.write(publicKeyBuffer, 0, rule.publicKey);
  // Field.
  const fieldBuffer = Buffer.alloc(32);
  fieldBuffer.write(rule.field);
  return Buffer.concat([headerBuffer, publicKeyBuffer, fieldBuffer]);
};

export const deserializePubkeyMatchV2 = (buffer: Buffer, offset = 0): PubkeyMatchRuleV2 => {
  // Skip rule header.
  offset += 8;
  // PublicKey.
  const publicKey = beetSolana.publicKey.read(buffer, offset);
  offset += 32;
  // Field.
  const field = buffer
    .subarray(offset, offset + 32)
    .toString()
    .replace(/\u0000/g, '');
  offset += 32;

  return { type: RuleTypeV2.PubkeyMatch, publicKey, field };
};
