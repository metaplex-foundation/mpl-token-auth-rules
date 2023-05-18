import { Context, Serializer, mapSerializer } from '@metaplex-foundation/umi';
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

export const getProgramOwnedTreeRuleV2Serializer = (
  context: Pick<Context, 'serializer'>
): Serializer<ProgramOwnedTreeRuleV2> => {
  const s = context.serializer;
  return wrapSerializerInRuleHeaderV2(
    context,
    RuleTypeV2.ProgramOwnedTree,
    s.struct([
      ['pubkeyField', s.string({ size: 32 })],
      ['proofField', s.string({ size: 32 })],
      [
        'root',
        mapSerializer(
          s.bytes({ size: 32 }),
          (v: number[]): Uint8Array => new Uint8Array(v),
          (v: Uint8Array): number[] => [...v]
        ),
      ],
    ])
  );
};
