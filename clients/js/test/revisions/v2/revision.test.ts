/* eslint-disable prefer-template */
import {
  base16,
  base58PublicKey,
  generateSigner,
} from '@metaplex-foundation/umi';
import test from 'ava';
import {
  RuleSetRevisionV1,
  RuleSetRevisionV2,
  additionalSignerV2,
  getRuleSetRevisionV2FromV1,
  getRuleSetRevisionSerializer,
} from '../../../src';
import { createUmiSync, toHex, toString32Hex } from '../../_setup';

test('serialize', async (t) => {
  const umi = createUmiSync();
  const owner = generateSigner(umi).publicKey;
  const publicKeyA = generateSigner(umi).publicKey;
  const publicKeyB = generateSigner(umi).publicKey;
  const revision: RuleSetRevisionV2 = {
    libVersion: 2,
    name: 'My Rule Set',
    owner: base58PublicKey(owner),
    operations: {
      deposit: additionalSignerV2(publicKeyA),
      withdraw: additionalSignerV2(publicKeyB),
    },
  };
  const serializedRevision = toHex(
    getRuleSetRevisionSerializer(umi).serialize(revision)
  );

  const expectedRuleA = '0100000020000000' + toHex(publicKeyA);
  const expectedRuleB = '0100000020000000' + toHex(publicKeyB);
  t.is(
    serializedRevision,
    '02000000' + // Rule Set Version
      '02000000' + // Number of operations/rules
      toHex(owner) + // Owner
      toString32Hex(umi, 'My Rule Set') + // Name
      toString32Hex(umi, 'deposit') + // Deposit operation
      toString32Hex(umi, 'withdraw') + // Withdraw operation
      expectedRuleA +
      expectedRuleB
  );
});

test('deserialize', async (t) => {
  const umi = createUmiSync();
  const owner = generateSigner(umi).publicKey;
  const publicKeyA = generateSigner(umi).publicKey;
  const publicKeyB = generateSigner(umi).publicKey;
  const ruleA = '0100000020000000' + toHex(publicKeyA);
  const ruleB = '0100000020000000' + toHex(publicKeyB);
  const buffer =
    '02000000' + // Rule Set Version
    '02000000' + // Number of operations/rules
    toHex(owner) + // Owner
    toString32Hex(umi, 'My Rule Set') + // Name
    toString32Hex(umi, 'deposit') + // Deposit operation
    toString32Hex(umi, 'withdraw') + // Withdraw operation
    ruleA +
    ruleB;
  const revision = getRuleSetRevisionSerializer(umi).deserialize(
    base16.serialize(buffer)
  )[0];
  t.deepEqual(revision, {
    libVersion: 2,
    name: 'My Rule Set',
    owner: base58PublicKey(owner),
    operations: {
      deposit: additionalSignerV2(publicKeyA),
      withdraw: additionalSignerV2(publicKeyB),
    },
  });
});

test('convert from v1', async (t) => {
  // Given a RuleSetRevisionV1.
  const umi = createUmiSync();
  const owner = generateSigner(umi).publicKey;
  const publicKeyA = generateSigner(umi).publicKey;
  const publicKeyB = generateSigner(umi).publicKey;
  const name = 'My Rule Set';
  const revisionV1: RuleSetRevisionV1 = {
    libVersion: 1,
    ruleSetName: name,
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

  // When we convert it to a RuleSetRevisionV2.
  const revisionV2 = getRuleSetRevisionV2FromV1(revisionV1);

  // Then we expect the following RuleSet data.
  t.deepEqual(revisionV2, <RuleSetRevisionV2>{
    libVersion: 2,
    name,
    owner: base58PublicKey(owner),
    operations: {
      deposit: additionalSignerV2(publicKeyA),
      withdraw: additionalSignerV2(publicKeyB),
    },
  });
});
