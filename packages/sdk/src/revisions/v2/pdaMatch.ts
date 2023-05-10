import { PublicKey } from '@solana/web3.js';
import { Base58PublicKey, toBase58PublicKey } from './base58PublicKey';
import {
  deserializePublicKey,
  deserializeString32,
  serializePublicKey,
  serializeString32,
} from './helpers';
import { serializeRuleHeaderV2 } from './rule';
import { RuleTypeV2 } from './ruleType';

export type PdaMatchRuleV2 = {
  type: 'PdaMatch';
  pdaField: string;
  program: Base58PublicKey;
  seedsField: string;
};

export const pdaMatchV2 = (
  pdaField: string,
  program: PublicKey | Base58PublicKey,
  seedsField: string,
): PdaMatchRuleV2 => ({
  type: 'PdaMatch',
  pdaField,
  program: toBase58PublicKey(program),
  seedsField,
});

export const serializePdaMatchV2 = (rule: PdaMatchRuleV2): Buffer => {
  return Buffer.concat([
    serializeRuleHeaderV2(RuleTypeV2.PdaMatch, 96),
    serializePublicKey(rule.program),
    serializeString32(rule.pdaField),
    serializeString32(rule.seedsField),
  ]);
};

export const deserializePdaMatchV2 = (buffer: Buffer, offset = 0): PdaMatchRuleV2 => {
  offset += 8; // Skip rule header.
  const program = deserializePublicKey(buffer, offset);
  offset += 32;
  const pdaField = deserializeString32(buffer, offset);
  offset += 32;
  const seedsField = deserializeString32(buffer, offset);
  offset += 32;
  return pdaMatchV2(pdaField, program, seedsField);
};
