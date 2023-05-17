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

export type ProgramOwnedListRuleV2 = {
  type: 'ProgramOwnedList';
  field: string;
  programs: Base58PublicKey[];
};

export const programOwnedListV2 = (
  field: string,
  programs: (PublicKey | Base58PublicKey)[],
): ProgramOwnedListRuleV2 => ({
  type: 'ProgramOwnedList',
  field,
  programs: programs.map(toBase58PublicKey),
});

export const serializeProgramOwnedListV2 = (rule: ProgramOwnedListRuleV2): Buffer => {
  const length = 32 + 32 * rule.programs.length;
  const headerBuffer = serializeRuleHeaderV2(RuleTypeV2.ProgramOwnedList, length);
  const fieldBuffer = serializeString32(rule.field);
  const publicKeyBuffers = rule.programs.map((publicKey) => serializePublicKey(publicKey));
  return Buffer.concat([headerBuffer, fieldBuffer, ...publicKeyBuffers]);
};

export const deserializeProgramOwnedListV2 = (
  buffer: Buffer,
  offset = 0,
): ProgramOwnedListRuleV2 => {
  // Header.
  const length = beet.u32.read(buffer, offset + 4);
  const numberOfPublicKeys = Math.floor((length - 32) / 32);
  offset += 8;

  // Field.
  const field = deserializeString32(buffer, offset);
  offset += 32;

  // PublicKeys.
  const programs = [];
  for (let index = 0; index < numberOfPublicKeys; index++) {
    programs.push(deserializePublicKey(buffer, offset));
    offset += 32;
  }

  return programOwnedListV2(field, programs);
};
