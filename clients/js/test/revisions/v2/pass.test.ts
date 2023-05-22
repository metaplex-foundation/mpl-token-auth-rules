/* eslint-disable prefer-template */
import { generateSigner, base58PublicKey } from '@metaplex-foundation/umi';
import test from 'ava';
import { RuleSetRevisionV2, isPassRuleV2, passV2 } from '../../../src';
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

test('isPassRuleV2', async (t) => {
  const umi = createUmiSync();
  const owner = generateSigner(umi).publicKey;
  const revision: RuleSetRevisionV2 = {
    libVersion: 2,
    name: 'My Rule Set',
    owner: base58PublicKey(owner),
    operations: {
      deposit: {
        type: 'Pass',
      },
    },
  };

  t.is(isPassRuleV2(revision.operations.deposit), true);
});
