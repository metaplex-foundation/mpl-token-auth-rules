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

export type ProgramOwnedTreeRuleV2 = {
  type: 'ProgramOwnedTree';
  pubkeyField: string;
  proofField: string;
  root: number[];
};

export const programOwnedTreeV2 = (
  pubkeyField: string,
  proofField: string,
  root: Uint8Array | number[]
): ProgramOwnedTreeRuleV2 => ({
  type: 'ProgramOwnedTree',
  pubkeyField,
  proofField,
  root: [...root],
});

export const getProgramOwnedTreeRuleV2Serializer =
  (): Serializer<ProgramOwnedTreeRuleV2> =>
    wrapSerializerInRuleHeaderV2(
      RuleTypeV2.ProgramOwnedTree,
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

export const isProgramOwnedTreeRuleV2 = (
  rule: RuleV1 | RuleV2
): rule is ProgramOwnedTreeRuleV2 =>
  isRuleV2(rule) && rule.type === 'ProgramOwnedTree';
