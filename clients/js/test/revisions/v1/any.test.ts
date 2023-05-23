/* eslint-disable prefer-template */
import { generateSigner } from '@metaplex-foundation/umi';
import test from 'ava';
import { RuleSetRevisionV1, isAnyRuleV1 } from '../../../src';
import { createUmiSync } from '../../_setup';

test('isAnyRuleV1', async (t) => {
  const umi = createUmiSync();
  const owner = generateSigner(umi).publicKey;
  const publicKeyA = generateSigner(umi).publicKey;
  const publicKeyB = generateSigner(umi).publicKey;
  const revision: RuleSetRevisionV1 = {
    libVersion: 1,
    ruleSetName: 'My Rule Set',
    owner: [...owner.bytes],
    operations: {
      deposit: {
        Any: {
          rules: [{ AdditionalSigner: { account: [...publicKeyA.bytes] } }],
        },
      },
      withdraw: {
        Any: {
          rules: [{ AdditionalSigner: { account: [...publicKeyB.bytes] } }],
        },
      },
    },
  };
  t.true(isAnyRuleV1(revision.operations.deposit));
});

test('not isAnyRuleV1', async (t) => {
  const umi = createUmiSync();
  const owner = generateSigner(umi).publicKey;
  const publicKeyA = generateSigner(umi).publicKey;
  const publicKeyB = generateSigner(umi).publicKey;
  const revision: RuleSetRevisionV1 = {
    libVersion: 1,
    ruleSetName: 'My Rule Set',
    owner: [...owner.bytes],
    operations: {
      deposit: {
        All: {
          rules: [{ AdditionalSigner: { account: [...publicKeyA.bytes] } }],
        },
      },
      withdraw: {
        Any: {
          rules: [{ AdditionalSigner: { account: [...publicKeyB.bytes] } }],
        },
      },
    },
  };

  t.false(isAnyRuleV1(revision.operations.deposit));
});
