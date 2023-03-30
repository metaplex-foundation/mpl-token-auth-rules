import * as beet from '@metaplex-foundation/beet';
import BN from 'bn.js';
import { deserializeString32, serializeString32 } from './helpers';
import { serializeRuleHeaderV2 } from './rule';
import { RuleTypeV2 } from './ruleType';

export enum AmountOperator {
  Lt, // Less Than
  LtEq, // Less Than or Equal To
  Eq, // Equal To
  GtEq, // Greater Than or Equal To
  Gt, // Greater Than
}

export type AmountOperatorString = '<' | '<=' | '=' | '>=' | '>';

export type AmountRuleV2 = {
  type: RuleTypeV2.Amount;
  field: string;
  operator: AmountOperator;
  amount: number | BN;
};

export const amountV2 = (
  field: string,
  operator: AmountOperator | AmountOperatorString,
  amount: number | BN,
): AmountRuleV2 => ({
  type: RuleTypeV2.Amount,
  field,
  operator: parseAmountOperator(operator),
  amount,
});

export const serializeAmountV2 = (rule: AmountRuleV2): Buffer => {
  const headerBuffer = serializeRuleHeaderV2(RuleTypeV2.Amount, 8 + 8 + 32);
  const amountAndOperatorBuffer = Buffer.alloc(16);
  beet.u64.write(amountAndOperatorBuffer, 0, rule.amount);
  beet.u64.write(amountAndOperatorBuffer, 8, rule.operator);
  const fieldBuffer = serializeString32(rule.field);
  return Buffer.concat([headerBuffer, amountAndOperatorBuffer, fieldBuffer]);
};

export const deserializeAmountV2 = (buffer: Buffer, offset = 0): AmountRuleV2 => {
  // Skip rule header.
  offset += 8;
  const amount = beet.u64.read(buffer, offset);
  offset += 8;
  const operator = Number(beet.u64.read(buffer, offset)) as AmountOperator;
  offset += 8;
  const field = deserializeString32(buffer, offset);
  return { type: RuleTypeV2.Amount, field, operator, amount };
};

const parseAmountOperator = (operator: AmountOperator | AmountOperatorString): AmountOperator => {
  return (
    {
      '<': AmountOperator.Lt,
      '<=': AmountOperator.LtEq,
      '=': AmountOperator.Eq,
      '>=': AmountOperator.GtEq,
      '>': AmountOperator.Gt,
    }[operator] ?? operator
  );
};
