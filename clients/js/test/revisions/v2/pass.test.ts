/* eslint-disable prefer-template */
import { generateSigner } from '@metaplex-foundation/umi';
import test from 'ava';
import { RuleSetRevisionV2, isPassRuleV2, passV2 } from '../../../src';
import {
  createUmiSync,
  deserializeRuleV2FromHex,
  serializeRuleV2AsHex,
} from '../../_setup';

test('serialize', async (t) => {
  const rule = passV2();
  const serializedRule = serializeRuleV2AsHex(rule);
  t.is(
    serializedRule,
    '09000000' + // Rule type (9)
      '00000000' // Rule length (0 bytes)
  );
});

test('deserialize', async (t) => {
  const buffer =
    '09000000' + // Rule type (9)
    '00000000'; // Rule length (0 bytes)
  const rule = deserializeRuleV2FromHex(buffer);
  t.deepEqual(rule, passV2());
});

test('isPassRuleV2', async (t) => {
  const umi = createUmiSync();
  const owner = generateSigner(umi).publicKey;
  const revision: RuleSetRevisionV2 = {
    libVersion: 2,
    name: 'My Rule Set',
    owner,
    operations: {
      deposit: {
        type: 'Pass',
      },
    },
  };

  t.true(isPassRuleV2(revision.operations.deposit));
});
