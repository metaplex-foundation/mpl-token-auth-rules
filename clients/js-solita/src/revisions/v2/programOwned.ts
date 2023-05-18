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

export type ProgramOwnedRuleV2 = {
  type: 'ProgramOwned';
  field: string;
  program: Base58PublicKey;
};

export const programOwnedV2 = (
  field: string,
  program: PublicKey | Base58PublicKey,
): ProgramOwnedRuleV2 => ({
  type: 'ProgramOwned',
  program: toBase58PublicKey(program),
  field,
});

export const serializeProgramOwnedV2 = (rule: ProgramOwnedRuleV2): Buffer => {
  return Buffer.concat([
    serializeRuleHeaderV2(RuleTypeV2.ProgramOwned, 64),
    serializePublicKey(rule.program),
    serializeString32(rule.field),
  ]);
};

export const deserializeProgramOwnedV2 = (buffer: Buffer, offset = 0): ProgramOwnedRuleV2 => {
  offset += 8; // Skip rule header.
  const program = deserializePublicKey(buffer, offset);
  offset += 32;
  const field = deserializeString32(buffer, offset);
  offset += 32;
  return programOwnedV2(field, program);
};
