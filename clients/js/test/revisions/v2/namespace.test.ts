/* eslint-disable prefer-template */
import test from 'ava';
import { namespaceV2 } from '../../../src';
import { deserializeRuleV2FromHex, serializeRuleV2AsHex } from '../../_setup';

test('serialize', async (t) => {
  const rule = namespaceV2();
  const serializedRule = serializeRuleV2AsHex(rule);
  t.is(
    serializedRule,
    '07000000' + // Rule type (7)
      '00000000' // Rule length (0 bytes)
  );
});

test('deserialize', async (t) => {
  const buffer =
    '07000000' + // Rule type (7)
    '00000000'; // Rule length (0 bytes)
  const rule = deserializeRuleV2FromHex(buffer);
  t.deepEqual(rule, namespaceV2());
});
