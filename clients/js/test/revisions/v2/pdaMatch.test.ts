/* eslint-disable prefer-template */
import { generateSigner, base58PublicKey } from '@metaplex-foundation/umi';
import test from 'ava';
import { RuleSetRevisionV2, isPdaMatchRuleV2, pdaMatchV2 } from '../../../src';
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
  const rule = pdaMatchV2('myAccount', program, 'mySeeds');
  const serializedRule = serializeRuleV2AsHex(umi, rule);
  t.is(
    serializedRule,
    '0a000000' + // Rule type (10)
      '60000000' + // Rule length (96 bytes)
      toHex(program) + // PublicKey
      toString32Hex(umi, 'myAccount') + // Pda Field
      toString32Hex(umi, 'mySeeds') // Seeds Field
  );
});

test('deserialize', async (t) => {
  const umi = createUmiSync();
  const program = generateSigner(umi).publicKey;
  const buffer =
    '0a000000' + // Rule type (10)
    '60000000' + // Rule length (96 bytes)
    toHex(program) + // PublicKey
    toString32Hex(umi, 'myAccount') + // Pda Field
    toString32Hex(umi, 'mySeeds'); // Seeds Field
  const rule = deserializeRuleV2FromHex(umi, buffer);
  t.deepEqual(rule, pdaMatchV2('myAccount', program, 'mySeeds'));
});

test('isPdaMatchRuleV2', async (t) => {
  const umi = createUmiSync();
  const owner = generateSigner(umi).publicKey;
  const program = generateSigner(umi).publicKey;
  const revision: RuleSetRevisionV2 = {
    libVersion: 2,
    name: 'My Rule Set',
    owner: base58PublicKey(owner),
    operations: {
      deposit: {
        type: 'PdaMatch',
        pdaField: 'myAccount',
        program: base58PublicKey(program),
        seedsField: 'mySeeds',
      },
    },
  };

  t.is(isPdaMatchRuleV2(revision.operations.deposit), true);
});