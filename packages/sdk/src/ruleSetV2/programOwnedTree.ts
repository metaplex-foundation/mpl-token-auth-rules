import { serializeRuleHeaderV2 } from './rule';
import { RuleTypeV2 } from './ruleType';

export type ProgramOwnedTreeRuleV2 = {
  type: RuleTypeV2.ProgramOwnedTree;
  pubkeyField: string;
  proofField: string;
  root: Uint8Array;
};

export const programOwnedTreeV2 = (
  pubkeyField: string,
  proofField: string,
  root: Uint8Array,
): ProgramOwnedTreeRuleV2 => ({
  type: RuleTypeV2.ProgramOwnedTree,
  pubkeyField,
  proofField,
  root,
});

export const serializeProgramOwnedTreeV2 = (rule: ProgramOwnedTreeRuleV2): Buffer => {
  const headerBuffer = serializeRuleHeaderV2(RuleTypeV2.ProgramOwnedTree, 96);

  // PubkeyField.
  const pubkeyFieldBuffer = Buffer.alloc(32);
  pubkeyFieldBuffer.write(rule.pubkeyField);

  // ProofField.
  const proofFieldBuffer = Buffer.alloc(32);
  proofFieldBuffer.write(rule.proofField);

  return Buffer.concat([headerBuffer, pubkeyFieldBuffer, proofFieldBuffer, rule.root]);
};

export const deserializeProgramOwnedTreeV2 = (buffer: Buffer, offset = 0): ProgramOwnedTreeRuleV2 => {
  // Skip rule header.
  offset += 8;
  // PubkeyField.
  const pubkeyField = buffer
    .subarray(offset, offset + 32)
    .toString()
    .replace(/\u0000/g, '');
  offset += 32;

  // ProofField.
  const proofField = buffer
    .subarray(offset, offset + 32)
    .toString()
    .replace(/\u0000/g, '');
  offset += 32;

  // Root.
  const root = new Uint8Array(buffer.subarray(offset, offset + 32));

  return { type: RuleTypeV2.ProgramOwnedTree, pubkeyField, proofField, root };
};
