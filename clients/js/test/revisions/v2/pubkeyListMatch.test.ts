/* eslint-disable prefer-template */
import { PublicKey, generateSigner, base58PublicKey } from '@metaplex-foundation/umi';
import test from 'ava';
import { RuleSetRevisionV2, isPubkeyListMatchRuleV2, pubkeyListMatchV2 } from '../../../src';
import {
  createUmiSync,
  deserializeRuleV2FromHex,
  serializeRuleV2AsHex,
  toHex,
  toString32Hex,
} from '../../_setup';

test('serialize', async (t) => {
  const umi = createUmiSync();
  const publicKeyA = generateSigner(umi).publicKey;
  const publicKeyB = generateSigner(umi).publicKey;
  const publicKeyC = generateSigner(umi).publicKey;
  const publicKeys: PublicKey[] = [publicKeyA, publicKeyB, publicKeyC];
  const rule = pubkeyListMatchV2('myAccount', publicKeys);
  const serializedRule = serializeRuleV2AsHex(umi, rule);
  t.is(
    serializedRule,
    '0e000000' + // Rule type
      '80000000' + // Rule length
      toString32Hex(umi, 'myAccount') + // Field
      toHex(publicKeyA) + // PublicKey A
      toHex(publicKeyB) + // PublicKey B
      toHex(publicKeyC) // PublicKey C
  );
});

test('deserialize', async (t) => {
  const umi = createUmiSync();
  const publicKeyA = generateSigner(umi).publicKey;
  const publicKeyB = generateSigner(umi).publicKey;
  const publicKeyC = generateSigner(umi).publicKey;
  const publicKeys: PublicKey[] = [publicKeyA, publicKeyB, publicKeyC];
  const buffer =
    '0e000000' + // Rule type
    '80000000' + // Rule length
    toString32Hex(umi, 'myAccount') + // Field
    toHex(publicKeyA) + // PublicKey A
    toHex(publicKeyB) + // PublicKey B
    toHex(publicKeyC); // PublicKey C
  const rule = deserializeRuleV2FromHex(umi, buffer);
  t.deepEqual(rule, pubkeyListMatchV2('myAccount', publicKeys));
});


test('isPubkeyListMatchRuleV2', async (t) => {
  const umi = createUmiSync();
  const owner = generateSigner(umi).publicKey;
  const publicKeyA = generateSigner(umi).publicKey;
  const publicKeyB = generateSigner(umi).publicKey;
  const revision: RuleSetRevisionV2 = {
    libVersion: 2,
    name: 'My Rule Set',
    owner: base58PublicKey(owner),
    operations: {
      deposit: {
        type: 'PubkeyListMatch',
        field: 'myField',
        publicKeys: [
          base58PublicKey(publicKeyA),
          base58PublicKey(publicKeyB),
        ],
      },
    },
  };
  t.is(isPubkeyListMatchRuleV2(revision.operations.deposit), true);
});
