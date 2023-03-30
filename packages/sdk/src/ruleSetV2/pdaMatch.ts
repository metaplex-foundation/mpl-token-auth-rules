import { PublicKey } from '@solana/web3.js';
import {
  deserializePublicKey,
  deserializeString32,
  serializePublicKey,
  serializeString32,
} from './helpers';
import { serializeRuleHeaderV2 } from './rule';
import { RuleTypeV2 } from './ruleType';

export type PdaMatchRuleV2 = {
  type: RuleTypeV2.PdaMatch;
  pdaField: string;
  program: PublicKey;
  seedsField: string;
};

export const pdaMatchV2 = (
  pdaField: string,
  program: PublicKey,
  seedsField: string,
): PdaMatchRuleV2 => ({
  type: RuleTypeV2.PdaMatch,
  pdaField,
  program,
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

  return { type: RuleTypeV2.PdaMatch, pdaField, program, seedsField };
};
