import test from 'ava';
import {
  AmountRuleV2,
  amountV2,
  deserializeRuleV2,
  serializeRuleV2,
} from '../../src/mpl-token-auth-rules';
import { serializeString32 } from '../../src/ruleSetV2/helpers';

test('serialize', async (t) => {
  const rule = amountV2('bananas', '>', 42);
  const serializedRule = serializeRuleV2(rule).toString('hex');
  t.is(
    serializedRule,
    '03000000' + // Rule type (3)
      '30000000' + // Rule length (48 bytes)
      '2a00000000000000' + // Amount (42)
      '0400000000000000' + // Operator (4)
      serializeString32('bananas').toString('hex'), // Field
  );
});

test('deserialize', async (t) => {
  const hexBuffer =
    '03000000' + // Rule type (3)
    '30000000' + // Rule length (48 bytes)
    '2a00000000000000' + // Amount (42)
    '0400000000000000' + // Operator (4)
    serializeString32('bananas').toString('hex'); // Field
  const buffer = Buffer.from(hexBuffer, 'hex');
  const rule = deserializeRuleV2(buffer) as AmountRuleV2;
  rule.amount = Number(rule.amount);
  t.deepEqual(rule, amountV2('bananas', '>', 42));
});
