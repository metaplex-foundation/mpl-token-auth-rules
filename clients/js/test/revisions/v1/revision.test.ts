/* eslint-disable prefer-template */
import { generateSigner, mergeBytes } from '@metaplex-foundation/umi';
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
    owner: [...owner.bytes],
    operations: {
      deposit: {
        AdditionalSigner: { account: [...publicKeyA.bytes] },
      },
      withdraw: {
        AdditionalSigner: { account: [...publicKeyB.bytes] },
      },
    },
  };
  const serializedRevision =
    getRuleSetRevisionSerializer(umi).serialize(revision);

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
    owner: [...owner.bytes],
    operations: {
      deposit: {
        AdditionalSigner: { account: [...publicKeyA.bytes] },
      },
      withdraw: {
        AdditionalSigner: { account: [...publicKeyB.bytes] },
      },
    },
  };
  const buffer = mergeBytes([new Uint8Array([1]), encode(revision)]);
  const deserializedRevision =
    getRuleSetRevisionSerializer(umi).deserialize(buffer)[0];
  t.deepEqual(deserializedRevision, revision);
});
