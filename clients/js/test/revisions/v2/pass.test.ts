/* eslint-disable prefer-template */
import test from 'ava';
import { passV2 } from '../../../src';
import {
  createUmiSync,
  deserializeRuleV2FromHex,
  serializeRuleV2AsHex,
} from '../../_setup';

test('serialize', async (t) => {
  const umi = createUmiSync();
  const rule = passV2();
  const serializedRule = serializeRuleV2AsHex(umi, rule);
  t.is(
    serializedRule,
    '09000000' + // Rule type (9)
      '00000000' // Rule length (0 bytes)
  );
});

test('deserialize', async (t) => {
  const umi = createUmiSync();
  const buffer =
    '09000000' + // Rule type (9)
    '00000000'; // Rule length (0 bytes)
  const rule = deserializeRuleV2FromHex(umi, buffer);
  t.deepEqual(rule, passV2());
});
