/* eslint-disable prefer-template */
import { generateSigner } from '@metaplex-foundation/umi';
import test from 'ava';
import { pubkeyMatchV2 } from '../../../src';
import {
  createUmiSync,
  deserializeRuleV2FromHex,
  serializeRuleV2AsHex,
  toHex,
  toString32Hex,
} from '../../_setup';

test('serialize', async (t) => {
  const umi = createUmiSync();
  const { publicKey } = generateSigner(umi);
  const rule = pubkeyMatchV2('myAccount', publicKey);
  const serializedRule = serializeRuleV2AsHex(rule);
  t.is(
    serializedRule,
    '0f000000' + // Rule type
      '40000000' + // Rule length
      toHex(publicKey) + // PublicKey
      toString32Hex('myAccount') // Field
  );
});

test('deserialize', async (t) => {
  const umi = createUmiSync();
  const { publicKey } = generateSigner(umi);
  const buffer =
    '0f000000' + // Rule type
    '40000000' + // Rule length
    toHex(publicKey) + // PublicKey
    toString32Hex('myAccount'); // Field
  const rule = deserializeRuleV2FromHex(buffer);
  t.deepEqual(rule, pubkeyMatchV2('myAccount', publicKey));
});
