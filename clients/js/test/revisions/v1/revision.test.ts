/* eslint-disable prefer-template */
import {
  generateSigner,
  mergeBytes,
  publicKeyBytes,
} from '@metaplex-foundation/umi';
import { encode } from '@msgpack/msgpack';
import test from 'ava';
import { RuleSetRevisionV1, getRuleSetRevisionSerializer } from '../../../src';
import { createUmiSync } from '../../_setup';

test('serialize', async (t) => {
  const umi = createUmiSync();
  const owner = generateSigner(umi).publicKey;
  const publicKeyA = generateSigner(umi).publicKey;
  const publicKeyB = generateSigner(umi).publicKey;
  const revision: RuleSetRevisionV1 = {
    libVersion: 1,
    ruleSetName: 'My Rule Set',
    owner: [...publicKeyBytes(owner)],
    operations: {
      deposit: {
        AdditionalSigner: { account: [...publicKeyBytes(publicKeyA)] },
      },
      withdraw: {
        AdditionalSigner: { account: [...publicKeyBytes(publicKeyB)] },
      },
    },
  };
  const serializedRevision = getRuleSetRevisionSerializer().serialize(revision);

  t.deepEqual(serializedRevision, encode(revision));
});

test('deserialize', async (t) => {
  const umi = createUmiSync();
  const owner = generateSigner(umi).publicKey;
  const publicKeyA = generateSigner(umi).publicKey;
  const publicKeyB = generateSigner(umi).publicKey;
  const revision: RuleSetRevisionV1 = {
    libVersion: 1,
    ruleSetName: 'My Rule Set',
    owner: [...publicKeyBytes(owner)],
    operations: {
      deposit: {
        AdditionalSigner: { account: [...publicKeyBytes(publicKeyA)] },
      },
      withdraw: {
        AdditionalSigner: { account: [...publicKeyBytes(publicKeyB)] },
      },
    },
  };
  const buffer = mergeBytes([new Uint8Array([1]), encode(revision)]);
  const deserializedRevision =
    getRuleSetRevisionSerializer().deserialize(buffer)[0];
  t.deepEqual(deserializedRevision, revision);
});
