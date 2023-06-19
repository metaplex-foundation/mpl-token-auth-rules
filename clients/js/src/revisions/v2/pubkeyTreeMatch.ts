import {
  Serializer,
  bytes,
  mapSerializer,
  string,
  struct,
} from '@metaplex-foundation/umi/serializers';
import type { RuleV1 } from '../v1';
import { RuleV2, isRuleV2 } from './rule';
import { wrapSerializerInRuleHeaderV2 } from './ruleHeader';
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
  root: Uint8Array | number[]
): PubkeyTreeMatchRuleV2 => ({
  type: 'PubkeyTreeMatch',
  pubkeyField,
  proofField,
  root: [...root],
});

export const getPubkeyTreeMatchRuleV2Serializer =
  (): Serializer<PubkeyTreeMatchRuleV2> =>
    wrapSerializerInRuleHeaderV2(
      RuleTypeV2.PubkeyTreeMatch,
      struct([
        ['pubkeyField', string({ size: 32 })],
        ['proofField', string({ size: 32 })],
        [
          'root',
          mapSerializer(
            bytes({ size: 32 }),
            (v: number[]): Uint8Array => new Uint8Array(v),
            (v: Uint8Array): number[] => [...v]
          ),
        ],
      ])
    );

export const isPubkeyTreeMatchRuleV2 = (
  rule: RuleV1 | RuleV2
): rule is PubkeyTreeMatchRuleV2 =>
  isRuleV2(rule) && rule.type === 'PubkeyTreeMatch';
