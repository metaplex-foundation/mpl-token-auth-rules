import * as beetSolana from '@metaplex-foundation/beet-solana';
import { PublicKey } from '@solana/web3.js';
import { serializeRuleHeaderV2 } from './rule';
import { RuleTypeV2 } from './ruleType';

export type PubkeyListMatchRuleV2 = {
  type: RuleTypeV2.PubkeyListMatch;
  field: string;
  publicKeys: PublicKey[];
};

export const pubkeyListMatchV2 = (
  field: string,
  publicKeys: PublicKey[],
): PubkeyListMatchRuleV2 => ({
  type: RuleTypeV2.PubkeyListMatch,
  field,
  publicKeys,
});

export const serializePubkeyListMatchV2 = (rule: PubkeyListMatchRuleV2): Buffer => {
  const headerBuffer = serializeRuleHeaderV2(
    RuleTypeV2.PubkeyListMatch,
    32 + 32 * rule.publicKeys.length,
  );
  // Field.
  const fieldBuffer = Buffer.alloc(32);
  fieldBuffer.write(rule.field);
  // PublicKeys.
  const publicKeysBuffer = Buffer.alloc(32 * rule.publicKeys.length);
  let offset = 0;
  rule.publicKeys.forEach((publicKey) => {
    beetSolana.publicKey.write(publicKeysBuffer, offset, publicKey);
    offset += 32;
  });
  return Buffer.concat([headerBuffer, fieldBuffer, publicKeysBuffer]);
};

export const deserializePubkeyListMatchV2 = (buffer: Buffer, offset = 0): PubkeyListMatchRuleV2 => {
  // Skip rule header.
  offset += 8;
  // Field.
  const field = buffer
    .subarray(offset, offset + 32)
    .toString()
    .replace(/\u0000/g, '');
  buffer = buffer.subarray(offset + 32);
  // PublicKeys.
  const publicKeys = [];
  while (buffer.length) {
    publicKeys.push(beetSolana.publicKey.read(buffer, 0));
    buffer = buffer.subarray(32);
  }
  return { type: RuleTypeV2.PubkeyListMatch, field, publicKeys };
};
