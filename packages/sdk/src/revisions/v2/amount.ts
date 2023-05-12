import * as beet from '@metaplex-foundation/beet';
import { deserializeString32, serializeString32 } from './helpers';
import { serializeRuleHeaderV2 } from './rule';
import { RuleTypeV2 } from './ruleType';
import {
  AmountOperator,
  AmountOperatorString,
  parseAmountOperator,
  parseAmountOperatorString,
} from '../shared';

export type AmountRuleV2 = {
  type: 'Amount';
  field: string;
  operator: AmountOperatorString;
  amount: number;
};

export const amountV2 = (
  field: string,
  operator: AmountOperator | AmountOperatorString,
  amount: number,
): AmountRuleV2 => ({
  type: 'Amount',
  field,
  operator: parseAmountOperatorString(operator),
  amount,
});

export const serializeAmountV2 = (rule: AmountRuleV2): Buffer => {
  const headerBuffer = serializeRuleHeaderV2(RuleTypeV2.Amount, 8 + 8 + 32);
  const amountAndOperatorBuffer = Buffer.alloc(16);
  beet.u64.write(amountAndOperatorBuffer, 0, rule.amount);
  beet.u64.write(amountAndOperatorBuffer, 8, parseAmountOperator(rule.operator));
  const fieldBuffer = serializeString32(rule.field);
  return Buffer.concat([headerBuffer, amountAndOperatorBuffer, fieldBuffer]);
};

export const deserializeAmountV2 = (buffer: Buffer, offset = 0): AmountRuleV2 => {
  offset += 8; // Skip rule header.
  const amount = beet.u64.read(buffer, offset);
  const amountAsNumber = typeof amount === 'number' ? amount : amount.toNumber();
  offset += 8;
  const operator = Number(beet.u64.read(buffer, offset)) as AmountOperator;
  offset += 8;
  const field = deserializeString32(buffer, offset);
  return amountV2(field, operator, amountAsNumber);
};
