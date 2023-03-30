import * as beetSolana from '@metaplex-foundation/beet-solana';
import { PublicKey } from '@solana/web3.js';
import { serializeRuleHeaderV2 } from './rule';
import { RuleTypeV2 } from './ruleType';

export type ProgramOwnedRuleV2 = {
  type: RuleTypeV2.ProgramOwned;
  program: PublicKey;
  field: string;
};

export const programOwnedV2 = (program: PublicKey, field: string): ProgramOwnedRuleV2 => ({
  type: RuleTypeV2.ProgramOwned,
  program,
  field,
});

export const serializeProgramOwnedV2 = (rule: ProgramOwnedRuleV2): Buffer => {
  const headerBuffer = serializeRuleHeaderV2(RuleTypeV2.ProgramOwned, 64);
  // PublicKey.
  const publicKeyBuffer = Buffer.alloc(32);
  beetSolana.publicKey.write(publicKeyBuffer, 0, rule.program);
  // Field.
  const fieldBuffer = Buffer.alloc(32);
  fieldBuffer.write(rule.field);
  return Buffer.concat([headerBuffer, publicKeyBuffer, fieldBuffer]);
};

export const deserializeProgramOwnedV2 = (buffer: Buffer, offset = 0): ProgramOwnedRuleV2 => {
  // Skip rule header.
  offset += 8;
  // PublicKey.
  const program = beetSolana.publicKey.read(buffer, offset);
  offset += 32;
  // Field.
  const field = buffer
    .subarray(offset, offset + 32)
    .toString()
    .replace(/\u0000/g, '');
  offset += 32;

  return { type: RuleTypeV2.ProgramOwned, program, field };
};
