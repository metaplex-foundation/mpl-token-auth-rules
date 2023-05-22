import { Context, Serializer, mapSerializer } from '@metaplex-foundation/umi';
import {
  AmountOperator,
  AmountOperatorString,
  toAmountOperator,
  toAmountOperatorString,
} from '../shared';
import { wrapSerializerInRuleHeaderV2 } from './ruleHeader';
import { RuleTypeV2 } from './ruleType';
import { RuleV1 } from '../v1';
import { RuleV2, isRuleV2 } from './rule';

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

export const getAmountRuleV2Serializer = (
  context: Pick<Context, 'serializer'>
): Serializer<AmountRuleV2> => {
  const s = context.serializer;
  return wrapSerializerInRuleHeaderV2(
    context,
    RuleTypeV2.Amount,
    s.struct([
      [
        'amount',
        mapSerializer(
          s.u64(),
          (v: number): number | bigint => v,
          (v: number | bigint): number => Number(v)
        ),
      ],
      [
        'operator',
        mapSerializer(
          s.u64(),
          (v: AmountOperatorString): number | bigint => toAmountOperator(v),
          (v: number | bigint): AmountOperatorString =>
            toAmountOperatorString(Number(v))
        ),
      ],
      ['field', s.string({ size: 32 })],
    ])
  );
};

export const isAmountRule = (rule: RuleV1 | RuleV2): rule is AmountRuleV2 =>
  isRuleV2(rule) && rule.type === 'Amount';
