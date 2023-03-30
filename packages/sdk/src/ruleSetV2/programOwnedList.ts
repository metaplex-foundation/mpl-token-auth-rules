import * as beetSolana from '@metaplex-foundation/beet-solana';
import { PublicKey } from '@solana/web3.js';
import { serializeRuleHeaderV2 } from './rule';
import { RuleTypeV2 } from './ruleType';

export type ProgramOwnedListRuleV2 = {
  type: RuleTypeV2.ProgramOwnedList;
  field: string;
  programs: PublicKey[];
};

export const programOwnedListV2 = (
  field: string,
  programs: PublicKey[],
): ProgramOwnedListRuleV2 => ({
  type: RuleTypeV2.ProgramOwnedList,
  field,
  programs,
});

export const serializeProgramOwnedListV2 = (rule: ProgramOwnedListRuleV2): Buffer => {
  const headerBuffer = serializeRuleHeaderV2(
    RuleTypeV2.ProgramOwnedList,
    32 + 32 * rule.programs.length,
  );
  // Field.
  const fieldBuffer = Buffer.alloc(32);
  fieldBuffer.write(rule.field);
  // PublicKeys.
  const publicKeysBuffer = Buffer.alloc(32 * rule.programs.length);
  let offset = 0;
  rule.programs.forEach((publicKey) => {
    beetSolana.publicKey.write(publicKeysBuffer, offset, publicKey);
    offset += 32;
  });
  return Buffer.concat([headerBuffer, fieldBuffer, publicKeysBuffer]);
};

export const deserializeProgramOwnedListV2 = (buffer: Buffer, offset = 0): ProgramOwnedListRuleV2 => {
  // Skip rule header.
  offset += 8;
  // Field.
  const field = buffer
    .subarray(offset, offset + 32)
    .toString()
    .replace(/\u0000/g, '');
  buffer = buffer.subarray(offset + 32);
  // PublicKeys.
  const programs = [];
  while (buffer.length) {
    programs.push(beetSolana.publicKey.read(buffer, 0));
    buffer = buffer.subarray(32);
  }
  return { type: RuleTypeV2.ProgramOwnedList, field, programs };
};
