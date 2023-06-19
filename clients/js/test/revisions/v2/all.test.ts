/* eslint-disable prefer-template */
import { generateSigner } from '@metaplex-foundation/umi';
import test from 'ava';
import {
  RuleSetRevisionV2,
  additionalSignerV2,
  allV2,
  isAllRuleV2,
} from '../../../src';
import {
  createUmiSync,
  deserializeRuleV2FromHex,
  serializeRuleV2AsHex,
  toHex,
} from '../../_setup';

test('serialize', async (t) => {
  const umi = createUmiSync();
  const publicKeyA = generateSigner(umi).publicKey;
  const publicKeyB = generateSigner(umi).publicKey;
  const rule = allV2([
    additionalSignerV2(publicKeyA),
    additionalSignerV2(publicKeyB),
  ]);
  const serializedRule = serializeRuleV2AsHex(rule);

  const expectedRuleA = `0100000020000000${toHex(publicKeyA)}`;
  const expectedRuleB = `0100000020000000${toHex(publicKeyB)}`;
  t.is(
    serializedRule,
    '02000000' + // Rule type
      '58000000' + // Rule length
      '0200000000000000' + // Number of rules
      expectedRuleA +
      expectedRuleB
  );
});

test('deserialize', async (t) => {
  const umi = createUmiSync();
  const publicKeyA = generateSigner(umi).publicKey;
  const publicKeyB = generateSigner(umi).publicKey;
  const ruleA = `0100000020000000${toHex(publicKeyA)}`;
  const ruleB = `0100000020000000${toHex(publicKeyB)}`;
  const buffer =
    '02000000' + // Rule type
    '58000000' + // Rule length
    '0200000000000000' + // Number of rules
    ruleA +
    ruleB;
  const rule = deserializeRuleV2FromHex(buffer);
  t.deepEqual(
    rule,
    allV2([additionalSignerV2(publicKeyA), additionalSignerV2(publicKeyB)])
  );
});

test('isAllRuleV2', async (t) => {
  const umi = createUmiSync();
  const owner = generateSigner(umi).publicKey;
  const publicKeyA = generateSigner(umi).publicKey;
  const revision: RuleSetRevisionV2 = {
    libVersion: 2,
    name: 'My Rule Set',
    owner,
    operations: {
      deposit: {
        type: 'All',
        rules: [{ type: 'AdditionalSigner', publicKey: publicKeyA }],
      },
    },
  };
  t.true(isAllRuleV2(revision.operations.deposit));
});
