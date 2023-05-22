/* eslint-disable prefer-template */
import { generateSigner, base58PublicKey } from '@metaplex-foundation/umi';
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
    owner: [...owner.bytes],
    operations: {
      deposit: {
        AdditionalSigner: {
          account: [...publicKeyA.bytes],
        },
      },
    },
  };

  t.is(isRuleV1(revision.operations.deposit), true);
});

test('isRuleV1 with Not V1 rule', async (t) => {
  const umi = createUmiSync();
  const owner = generateSigner(umi).publicKey;
  const publicKeyA = generateSigner(umi).publicKey;
  const revision: RuleSetRevisionV1 = {
    libVersion: 1,
    ruleSetName: 'My Rule Set',
    owner: [...owner.bytes],
    operations: {
      deposit: {
        Not: {
          rule: {
            AdditionalSigner: {
              account: [...publicKeyA.bytes],
            },
          },
        },
      },
    },
  };

  t.is(isRuleV1(revision.operations.deposit), true);
});

test('isRuleV1 with Namespace V1 rule', async (t) => {
  const umi = createUmiSync();
  const owner = generateSigner(umi).publicKey;
  const revision: RuleSetRevisionV1 = {
    libVersion: 1,
    ruleSetName: 'My Rule Set',
    owner: [...owner.bytes],
    operations: {
      deposit: 'Namespace',
    },
  };

  t.is(isRuleV1(revision.operations.deposit), true);
});

test('isRuleV1 with a Not V2 rule', async (t) => {
  const umi = createUmiSync();
  const owner = generateSigner(umi).publicKey;
  const publicKeyA = generateSigner(umi).publicKey;
  const revision: RuleSetRevisionV2 = {
    libVersion: 2,
    name: 'My Rule Set',
    owner: base58PublicKey(owner),
    operations: {
      deposit: {
        type: 'Not',
        rule: {
          type: 'AdditionalSigner',
          publicKey: base58PublicKey(publicKeyA),
        },
      },
    },
  };

  t.is(isRuleV1(revision.operations.deposit), false);
});
