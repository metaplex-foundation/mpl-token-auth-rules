/* eslint-disable prefer-template */
import { PublicKey, generateSigner } from '@metaplex-foundation/umi';
import test from 'ava';
import { programOwnedListV2 } from '../../../src';
import {
  createUmiSync,
  deserializeRuleV2FromHex,
  serializeRuleV2AsHex,
  toHex,
  toString32Hex,
} from '../../_setup';

test('serialize', async (t) => {
  const umi = createUmiSync();
  const programA = generateSigner(umi).publicKey;
  const programB = generateSigner(umi).publicKey;
  const programs: PublicKey[] = [programA, programB];
  const rule = programOwnedListV2('myAccount', programs);
  const serializedRule = serializeRuleV2AsHex(umi, rule);
  t.is(
    serializedRule,
    '0c000000' + // Rule type
      '60000000' + // Rule length
      toString32Hex(umi, 'myAccount') + // Field
      toHex(programA) + // Program A
      toHex(programB) // Program B
  );
});

test('deserialize', async (t) => {
  const umi = createUmiSync();
  const programA = generateSigner(umi).publicKey;
  const programB = generateSigner(umi).publicKey;
  const programs: PublicKey[] = [programA, programB];
  const buffer =
    '0c000000' + // Rule type
    '60000000' + // Rule length
    toString32Hex(umi, 'myAccount') + // Field
    toHex(programA) + // Program A
    toHex(programB); // Program B
  const rule = deserializeRuleV2FromHex(umi, buffer);
  t.deepEqual(rule, programOwnedListV2('myAccount', programs));
});
