import { deserializeString32, serializeString32 } from './helpers';
import { serializeRuleHeaderV2 } from './rule';
import { RuleTypeV2 } from './ruleType';

export type PubkeyTreeMatchRuleV2 = {
  type: 'PubkeyTreeMatch';
  pubkeyField: string;
  proofField: string;
  root: number[];
};

export const pubkeyTreeMatchV2 = (
  pubkeyField: string,
  proofField: string,
  root: Uint8Array | number[],
): PubkeyTreeMatchRuleV2 => ({ type: 'PubkeyTreeMatch', pubkeyField, proofField, root: [...root] });

export const serializePubkeyTreeMatchV2 = (rule: PubkeyTreeMatchRuleV2): Buffer => {
  return Buffer.concat([
    serializeRuleHeaderV2(RuleTypeV2.PubkeyTreeMatch, 96),
    serializeString32(rule.pubkeyField),
    serializeString32(rule.proofField),
    new Uint8Array(rule.root),
  ]);
};

export const deserializePubkeyTreeMatchV2 = (buffer: Buffer, offset = 0): PubkeyTreeMatchRuleV2 => {
  offset += 8; // Skip rule header.
  const pubkeyField = deserializeString32(buffer, offset);
  offset += 32;
  const proofField = deserializeString32(buffer, offset);
  offset += 32;
  const root = new Uint8Array(buffer.subarray(offset, offset + 32));

  return pubkeyTreeMatchV2(pubkeyField, proofField, root);
};
