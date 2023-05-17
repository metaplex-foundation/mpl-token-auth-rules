import { Context, Serializer, mergeBytes } from '@metaplex-foundation/umi';
import { RuleTypeV2, getRuleTypeV2AsString } from './ruleType';

export type RuleHeaderV2 = { type: number; length: number };

export const getRuleHeaderV2Serializer = (
  context: Pick<Context, 'serializer'>
): Serializer<RuleHeaderV2> => {
  const s = context.serializer;
  return s.struct([
    ['type', s.u32()],
    ['length', s.u32()],
  ]);
};

export const wrapSerializerInRuleHeaderV2 = <T extends { type: string }>(
  context: Pick<Context, 'serializer'>,
  type: RuleTypeV2,
  serializer: Serializer<Omit<T, 'type'>>
): Serializer<T> => {
  const typeAsString = getRuleTypeV2AsString(type);
  const headerSerializer = getRuleHeaderV2Serializer(context);
  return {
    description: typeAsString,
    fixedSize: serializer.fixedSize === null ? null : serializer.fixedSize + 8,
    maxSize: serializer.maxSize === null ? null : serializer.maxSize + 8,
    serialize: (rule: T): Uint8Array => {
      const serializedRule = serializer.serialize(rule);
      const serializedHeader = headerSerializer.serialize({
        type,
        length: serializedRule.length,
      });
      return mergeBytes([serializedHeader, serializedRule]);
    },
    deserialize: (buffer: Uint8Array, offset = 0): [T, number] => {
      const [rule, ruleOffset] = serializer.deserialize(buffer, offset + 8);
      return [{ ...rule, type: typeAsString } as T, ruleOffset];
    },
  };
};
