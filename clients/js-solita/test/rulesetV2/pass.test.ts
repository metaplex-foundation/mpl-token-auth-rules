import test from 'ava';
import { deserializeRuleV2, passV2, serializeRuleV2 } from '../../src';

test('serialize', async (t) => {
  const rule = passV2();
  const serializedRule = serializeRuleV2(rule).toString('hex');
  t.is(
    serializedRule,
    '09000000' + // Rule type (9)
      '00000000', // Rule length (0 bytes)
  );
});

test('deserialize', async (t) => {
  const hexBuffer =
    '09000000' + // Rule type (9)
    '00000000'; // Rule length (0 bytes)
  const buffer = Buffer.from(hexBuffer, 'hex');
  const rule = deserializeRuleV2(buffer);
  t.deepEqual(rule, passV2());
});
