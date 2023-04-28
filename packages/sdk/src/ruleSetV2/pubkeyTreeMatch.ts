import { deserializeString32, serializeString32 } from './helpers';
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
  return Buffer.concat([
    serializeRuleHeaderV2(RuleTypeV2.PubkeyTreeMatch, 96),
    serializeString32(rule.pubkeyField),
    serializeString32(rule.proofField),
    rule.root,
  ]);
};

export const deserializePubkeyTreeMatchV2 = (buffer: Buffer, offset = 0): PubkeyTreeMatchRuleV2 => {
  offset += 8; // Skip rule header.
  const pubkeyField = deserializeString32(buffer, offset);
  offset += 32;
  const proofField = deserializeString32(buffer, offset);
  offset += 32;
  const root = new Uint8Array(buffer.subarray(offset, offset + 32));

  return { type: RuleTypeV2.PubkeyTreeMatch, pubkeyField, proofField, root };
};
