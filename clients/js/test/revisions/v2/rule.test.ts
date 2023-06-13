/* eslint-disable prefer-template */
import { generateSigner, publicKeyBytes } from '@metaplex-foundation/umi';
import test from 'ava';
import { RuleSetRevisionV2, isRuleV2, RuleSetRevisionV1 } from '../../../src';
import { createUmiSync } from '../../_setup';

test('isRuleV2 with AdditionalSigner V2 rule', async (t) => {
  const umi = createUmiSync();
  const owner = generateSigner(umi).publicKey;
  const publicKeyA = generateSigner(umi).publicKey;
  const revision: RuleSetRevisionV2 = {
    libVersion: 2,
    name: 'My Rule Set',
    owner,
    operations: {
      deposit: {
        type: 'AdditionalSigner',
        publicKey: publicKeyA,
      },
    },
  };

  t.is(isRuleV2(revision.operations.deposit), true);
});

test('isRuleV2 with Not V2 rule', async (t) => {
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

  t.true(isRuleV2(revision.operations.deposit));
});

test('isRuleV2 with Pass V2 rule', async (t) => {
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

  t.true(isRuleV2(revision.operations.deposit));
});

test('isRuleV1 with a Not V2 rule', async (t) => {
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

  t.false(isRuleV2(revision.operations.deposit));
});
