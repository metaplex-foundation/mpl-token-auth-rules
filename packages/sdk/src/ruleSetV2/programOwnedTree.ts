import { deserializeString32, serializeString32 } from './helpers';
import { serializeRuleHeaderV2 } from './rule';
import { RuleTypeV2 } from './ruleType';

export type ProgramOwnedTreeRuleV2 = {
  type: 'ProgramOwnedTree';
  pubkeyField: string;
  proofField: string;
  root: Uint8Array;
};

export const programOwnedTreeV2 = (
  pubkeyField: string,
  proofField: string,
  root: Uint8Array,
): ProgramOwnedTreeRuleV2 => ({ type: 'ProgramOwnedTree', pubkeyField, proofField, root });

export const serializeProgramOwnedTreeV2 = (rule: ProgramOwnedTreeRuleV2): Buffer => {
  return Buffer.concat([
    serializeRuleHeaderV2(RuleTypeV2.ProgramOwnedTree, 96),
    serializeString32(rule.pubkeyField),
    serializeString32(rule.proofField),
    rule.root,
  ]);
};

export const deserializeProgramOwnedTreeV2 = (
  buffer: Buffer,
  offset = 0,
): ProgramOwnedTreeRuleV2 => {
  offset += 8; // Skip rule header.
  const pubkeyField = deserializeString32(buffer, offset);
  offset += 32;
  const proofField = deserializeString32(buffer, offset);
  offset += 32;
  const root = new Uint8Array(buffer.subarray(offset, offset + 32));

  return programOwnedTreeV2(pubkeyField, proofField, root);
};
