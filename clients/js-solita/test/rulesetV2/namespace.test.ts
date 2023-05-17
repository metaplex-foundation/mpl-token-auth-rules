import test from 'ava';
import { deserializeRuleV2, namespaceV2, serializeRuleV2 } from '../../src';

test('serialize', async (t) => {
  const rule = namespaceV2();
  const serializedRule = serializeRuleV2(rule).toString('hex');
  t.is(
    serializedRule,
    '07000000' + // Rule type (7)
      '00000000', // Rule length (0 bytes)
  );
});

test('deserialize', async (t) => {
  const hexBuffer =
    '07000000' + // Rule type (7)
    '00000000'; // Rule length (0 bytes)
  const buffer = Buffer.from(hexBuffer, 'hex');
  const rule = deserializeRuleV2(buffer);
  t.deepEqual(rule, namespaceV2());
});
