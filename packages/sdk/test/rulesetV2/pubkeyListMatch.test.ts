import { Keypair, PublicKey } from '@solana/web3.js';
import test from 'ava';
import {
  deserializeRuleV2,
  RuleTypeV2,
  serializeRuleV2,
  pubkeyListMatchV2,
} from '../../src/mpl-token-auth-rules';
import { serializeString32 } from '../../src/ruleSetV2/helpers';

test('serialize', async (t) => {
  const publicKeyA = Keypair.generate().publicKey;
  const publicKeyB = Keypair.generate().publicKey;
  const publicKeyC = Keypair.generate().publicKey;
  const publicKeys: PublicKey[] = [publicKeyA, publicKeyB, publicKeyC];
  const rule = pubkeyListMatchV2('myAccount', publicKeys);
  const serializedRule = serializeRuleV2(rule).toString('hex');
  t.is(
    serializedRule,
    '0e000000' + // Rule type
      '80000000' + // Rule length
      serializeString32('myAccount').toString('hex') + // Field
      publicKeyA.toBuffer().toString('hex') + // PublicKey A
      publicKeyB.toBuffer().toString('hex') + // PublicKey B
      publicKeyC.toBuffer().toString('hex'), // PublicKey C
  );
});

test('deserialize', async (t) => {
  const publicKeyA = Keypair.generate().publicKey;
  const publicKeyB = Keypair.generate().publicKey;
  const publicKeyC = Keypair.generate().publicKey;
  const publicKeys: PublicKey[] = [publicKeyA, publicKeyB, publicKeyC];
  const hexBuffer =
    '0e000000' + // Rule type
    '80000000' + // Rule length
    serializeString32('myAccount').toString('hex') + // Field
    publicKeyA.toBuffer().toString('hex') + // PublicKey A
    publicKeyB.toBuffer().toString('hex') + // PublicKey B
    publicKeyC.toBuffer().toString('hex'); // PublicKey C
  const buffer = Buffer.from(hexBuffer, 'hex');
  const rule = deserializeRuleV2(buffer);
  t.deepEqual(rule, {
    type: RuleTypeV2.PubkeyListMatch,
    field: 'myAccount',
    publicKeys,
  });
});
