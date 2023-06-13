/* eslint-disable prefer-template */
import { generateSigner, publicKeyBytes } from '@metaplex-foundation/umi';
import test from 'ava';
import { RuleSetRevisionV1, RuleSetRevisionV2, isRuleV1 } from '../../../src';
import { createUmiSync } from '../../_setup';

test('isRuleV1 with AdditionalSigner V1 rule', async (t) => {
  const umi = createUmiSync();
  const owner = generateSigner(umi).publicKey;
  const publicKeyA = generateSigner(umi).publicKey;
  const revision: RuleSetRevisionV1 = {
    libVersion: 1,
    ruleSetName: 'My Rule Set',
    owner: [...publicKeyBytes(owner)],
    operations: {
      deposit: {
        AdditionalSigner: {
          account: [...publicKeyBytes(publicKeyA)],
        },
      },
    },
  };

  t.true(isRuleV1(revision.operations.deposit));
});

test('isRuleV1 with Not V1 rule', async (t) => {
  const umi = createUmiSync();
  const owner = generateSigner(umi).publicKey;
  const publicKeyA = generateSigner(umi).publicKey;
  const revision: RuleSetRevisionV1 = {
    libVersion: 1,
    ruleSetName: 'My Rule Set',
    owner: [...publicKeyBytes(owner)],
    operations: {
      deposit: {
        Not: {
          rule: {
            AdditionalSigner: {
              account: [...publicKeyBytes(publicKeyA)],
            },
          },
        },
      },
    },
  };

  t.true(isRuleV1(revision.operations.deposit));
});

test('isRuleV1 with Namespace V1 rule', async (t) => {
  const umi = createUmiSync();
  const owner = generateSigner(umi).publicKey;
  const revision: RuleSetRevisionV1 = {
    libVersion: 1,
    ruleSetName: 'My Rule Set',
    owner: [...publicKeyBytes(owner)],
    operations: {
      deposit: 'Namespace',
    },
  };

  t.true(isRuleV1(revision.operations.deposit));
});

test('isRuleV1 with a Not V2 rule', async (t) => {
  const umi = createUmiSync();
  const owner = generateSigner(umi).publicKey;
  const publicKeyA = generateSigner(umi).publicKey;
  const revision: RuleSetRevisionV2 = {
    libVersion: 2,
    name: 'My Rule Set',
    owner,
    operations: {
      deposit: {
        type: 'Not',
        rule: {
          type: 'AdditionalSigner',
          publicKey: publicKeyA,
        },
      },
    },
  };

  t.false(isRuleV1(revision.operations.deposit));
});
