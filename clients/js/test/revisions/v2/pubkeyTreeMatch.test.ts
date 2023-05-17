/* eslint-disable prefer-template */
import test from 'ava';
import { pubkeyTreeMatchV2 } from '../../../src';
import {
  createUmiSync,
  deserializeRuleV2FromHex,
  serializeRuleV2AsHex,
  toHex,
  toString32Hex,
} from '../../_setup';

test('serialize', async (t) => {
  const umi = createUmiSync();
  const root = new Uint8Array(
    [...Array(32)].map(() => Math.floor(Math.random() * 40))
  );
  const rule = pubkeyTreeMatchV2('myAccount', 'myProof', root);
  const serializedRule = serializeRuleV2AsHex(umi, rule);
  t.is(
    serializedRule,
    `10000000` + // Rule type
      '60000000' + // Rule length
      toString32Hex(umi, 'myAccount') + // pubkeyField
      toString32Hex(umi, 'myProof') + // proofField
      toHex(root) // root
  );
});

test('deserialize', async (t) => {
  const umi = createUmiSync();
  const root = new Uint8Array(
    [...Array(32)].map(() => Math.floor(Math.random() * 40))
  );
  const buffer =
    `10000000` + // Rule type
    '60000000' + // Rule length
    toString32Hex(umi, 'myAccount') + // pubkeyField
    toString32Hex(umi, 'myProof') + // proofField
    toHex(root); // root

  const rule = deserializeRuleV2FromHex(umi, buffer);
  t.deepEqual(rule, pubkeyTreeMatchV2('myAccount', 'myProof', root));
});
