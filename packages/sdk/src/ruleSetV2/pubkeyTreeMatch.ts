import { serializeRuleHeaderV2 } from './rule';
import { RuleTypeV2 } from './ruleType';

export type PubkeyTreeMatchRuleV2 = {
  type: RuleTypeV2.PubkeyTreeMatch;
  pubkeyField: string;
  proofField: string;
  root: Uint8Array;
};

export const pubkeyTreeMatchV2 = (
  pubkeyField: string,
  proofField: string,
  root: Uint8Array,
): PubkeyTreeMatchRuleV2 => ({
  type: RuleTypeV2.PubkeyTreeMatch,
  pubkeyField,
  proofField,
  root,
});

export const serializePubkeyTreeMatchV2 = (rule: PubkeyTreeMatchRuleV2): Buffer => {
  const headerBuffer = serializeRuleHeaderV2(RuleTypeV2.PubkeyTreeMatch, 96);

  // PubkeyField.
  const pubkeyFieldBuffer = Buffer.alloc(32);
  pubkeyFieldBuffer.write(rule.pubkeyField);

  // ProofField.
  const proofFieldBuffer = Buffer.alloc(32);
  proofFieldBuffer.write(rule.proofField);

  return Buffer.concat([headerBuffer, pubkeyFieldBuffer, proofFieldBuffer, rule.root]);
};

export const deserializePubkeyTreeMatchV2 = (buffer: Buffer, offset = 0): PubkeyTreeMatchRuleV2 => {
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

  return { type: RuleTypeV2.PubkeyTreeMatch, pubkeyField, proofField, root };
};
