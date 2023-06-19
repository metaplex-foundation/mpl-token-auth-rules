import {
  Serializer,
  mapSerializer,
  string,
  struct,
  u64,
} from '@metaplex-foundation/umi/serializers';
import {
  AmountOperator,
  AmountOperatorString,
  toAmountOperator,
  toAmountOperatorString,
} from '../shared';
import type { RuleV1 } from '../v1';
import { RuleV2, isRuleV2 } from './rule';
import { wrapSerializerInRuleHeaderV2 } from './ruleHeader';
import { RuleTypeV2 } from './ruleType';

export type AmountRuleV2 = {
  type: 'Amount';
  field: string;
  operator: AmountOperatorString;
  amount: number;
};

export const amountV2 = (
  field: string,
  operator: AmountOperator | AmountOperatorString,
  amount: number
): AmountRuleV2 => ({
  type: 'Amount',
  field,
  operator: toAmountOperatorString(operator),
  amount,
});

export const getAmountRuleV2Serializer = (): Serializer<AmountRuleV2> =>
  wrapSerializerInRuleHeaderV2(
    RuleTypeV2.Amount,
    struct([
      [
        'amount',
        mapSerializer(
          u64(),
          (v: number): number | bigint => v,
          (v: number | bigint): number => Number(v)
        ),
      ],
      [
        'operator',
        mapSerializer(
          u64(),
          (v: AmountOperatorString): number | bigint => toAmountOperator(v),
          (v: number | bigint): AmountOperatorString =>
            toAmountOperatorString(Number(v))
        ),
      ],
      ['field', string({ size: 32 })],
    ])
  );

export const isAmountRuleV2 = (rule: RuleV1 | RuleV2): rule is AmountRuleV2 =>
  isRuleV2(rule) && rule.type === 'Amount';
