/* eslint-disable prefer-template */
import { generateSigner } from '@metaplex-foundation/umi';
import test from 'ava';
import { programOwnedV2 } from '../../../src';
import {
  createUmiSync,
  deserializeRuleV2FromHex,
  serializeRuleV2AsHex,
  toHex,
  toString32Hex,
} from '../../_setup';

test('serialize', async (t) => {
  const umi = createUmiSync();
  const program = generateSigner(umi).publicKey;
  const rule = programOwnedV2('myAccount', program);
  const serializedRule = serializeRuleV2AsHex(rule);
  t.is(
    serializedRule,
    '0b000000' + // Rule type
      '40000000' + // Rule length
      toHex(program) + // PublicKey
      toString32Hex('myAccount') // Field
  );
});

test('deserialize', async (t) => {
  const umi = createUmiSync();
  const program = generateSigner(umi).publicKey;
  const buffer =
    '0b000000' + // Rule type
    '40000000' + // Rule length
    toHex(program) + // PublicKey
    toString32Hex('myAccount'); // Field
  const rule = deserializeRuleV2FromHex(buffer);
  t.deepEqual(rule, programOwnedV2('myAccount', program));
});
