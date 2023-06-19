/* eslint-disable prefer-template */
import test from 'ava';
import { amountV2 } from '../../../src';
import {
  deserializeRuleV2FromHex,
  serializeRuleV2AsHex,
  toString32Hex,
} from '../../_setup';

test('serialize', async (t) => {
  const rule = amountV2('bananas', '>', 42);
  const serializedRule = serializeRuleV2AsHex(rule);
  t.is(
    serializedRule,
    '03000000' + // Rule type (3)
      '30000000' + // Rule length (48 bytes)
      '2a00000000000000' + // Amount (42)
      '0400000000000000' + // Operator (4)
      toString32Hex('bananas') // Field
  );
});

test('deserialize', async (t) => {
  const buffer =
    '03000000' + // Rule type (3)
    '30000000' + // Rule length (48 bytes)
    '2a00000000000000' + // Amount (42)
    '0400000000000000' + // Operator (4)
    toString32Hex('bananas'); // Field
  const rule = deserializeRuleV2FromHex(buffer);
  t.deepEqual(rule, amountV2('bananas', '>', 42));
});
